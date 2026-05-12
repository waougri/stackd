import json
import pprint
from pathlib import Path

import pathspec
from checksumdir import dirhash
from paramiko.client import SSHClient

PROJECT_DIRS = ["stackd-be"]
SERVER_URL = "http://192.168.1.126:3000/"
checksum_f = Path("./checksums.json")


def load_gitignore(dir: Path) -> pathspec.PathSpec:
    gitignore = dir / ".gitignore"
    if gitignore.exists():
        patterns = gitignore.read_text().splitlines()
    else:
        patterns = []
    return pathspec.PathSpec.from_lines("gitwildmatch", patterns)


def upload_dir(sftp, local_dir, remote_dir):
    local_dir = Path(local_dir)
    spec = load_gitignore(local_dir)

    def mkdir_p(remote_path):
        """Recursively create remote directories."""
        parts = remote_path.split("/")
        for i in range(2, len(parts) + 1):
            path = "/".join(parts[:i])
            try:
                sftp.mkdir(path)
            except OSError:
                pass  # already exists

    mkdir_p(remote_dir)

    for item in local_dir.rglob("*"):
        rel = item.relative_to(local_dir)
        if spec.match_file(str(rel)):
            continue
        remote_file = f"{remote_dir}/{rel.as_posix()}"
        if item.is_dir():
            mkdir_p(remote_file)
        else:
            mkdir_p(remote_file.rsplit("/", 1)[0])  # ensure parent exists
            sftp.put(str(item), remote_file)
            print(f"  {rel}")


def deploy():

    client = SSHClient()
    client.load_system_host_keys()
    client.connect(
        "192.168.1.126",
        username="nebula",
        key_filename=str(Path.home() / ".ssh" / "id_ed25519"),
    )

    sftp = client.open_sftp()
    upload_dir(sftp, PROJECT_DIRS[0], "/home/nebula/backend")
    sftp.close()

    client.exec_command("pkill -f 'cargo run' || true")
    client.exec_command(
        "pkill -f 'target/release/stackd' || true"
    )  # the actual binary name

    # give it a moment to die
    import time

    time.sleep(1)

    # then start fresh
    stdin, stdout, stderr = client.exec_command(
        "cd /home/nebula/backend && source $HOME/.cargo/env && nohup cargo run --release > out.log 2>&1 &"
    )
    assert stdout.channel.recv_exit_status() == 0


def load_and_checksums():
    checksums = {}
    if not checksum_f.exists() or checksum_f.stat().st_size == 0:
        checksums = {pj: dirhash(pj, "sha256") for pj in PROJECT_DIRS}
        pprint.pprint(checksums)
        total_hash = dirhash(Path.cwd(), "sha256")

        with checksum_f.open("w", encoding="utf-8") as f:
            json.dump(checksums, f, indent=4, ensure_ascii=False)

        pprint.pprint(f"new deploy: {total_hash}")
        return

    else:
        with checksum_f.open("r", encoding="utf-8") as f:
            checksums: dict[str, str] = json.load(f)
            pprint.pprint(f"data found: {checksums}")
        be_chk = checksums[PROJECT_DIRS[0]]
        current_chk = dirhash(PROJECT_DIRS[0], "sha256")
        if be_chk == current_chk:
            print("No changes to deploy")
            exit(0)
        else:
            return


load_and_checksums()
deploy()
