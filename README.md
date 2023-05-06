# Redstone

Self-hosted CLI backup tool, made for personal files.

Redstone is an incremental backup application, no need to reupload unchanged files.


This is the client repository. If you’re looking for the server [click here](https://github.com/hammsvietro/redstone_server).

---
# Installation

As of now, the instalation script is only available on linux using systemD,
but the code is written to work with all major operating systems.


```bash
git clone https://github.com/hammsvietro/redstone.git
./install-linux.sh
systemctl enable redstone.service --user --now
```

#### Requirements
* Rust
* Cargo

# Usage

All the available commands can be viewed with:
```bash
redstone --help
```

After installing the client, you should connect to the [redstone server](https://github.com/hammsvietro/redstone_server) and login.
The commands for those actions are listed below.

## Commands

### Server config
Configure which server the client points to.
```bash
# redstone server-config <ADDRESS> [--port 80 --use-https]
$ redstone server-config 192.168.0.67 --port 4000

```
The address should contain only the hostname (do not specify protocols nor ports)

### Login

```bash
$ redstone auth
```

Accounts cannot be created via CLI, only in the web page.

### Track
Create a new backup and store the data in the server.
```bash
# redstone track <backup-name> [path]
$ redstone track my_first_backup .
```

The specified directory (current by default)  will be scanned recursively and all files will be sent to the server

A `.rsignore` file can be used to ignore folder and files, similar to `.gitignore`

### Clone
Create a copy of a existing backup in the current directory.
```bash
# redstone clone <backup-name>
$ redstone clone my_first_backup
```

All the data stored in the server will be pulled, similar to `git clone` 

### Status
Display all new, changed and deleted files.
```bash
$ redstone status
```

Similar to `git status`

### Pull
Pull latest changes from the server.
```bash
$ redstone pull
```

### Push
Push changes to the server.
```bash
$ redstone push
```

Pushing data is only allowed when the local files are up to date with the server.

# Contributing
Contributions and suggestions are very welcome! Feel free to open an issue.

# License
Redstone source code is licensed under the [GPL-3.0 License](LICENSE).
