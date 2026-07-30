


---

# dnsrs

A minimal, fast DNS manager for Linux written in Rust.
Supports profiles, history tracking, undo, and interactive CLI input.

---

## ✨ Features

* Create named DNS profiles
* Apply DNS profiles to your system
* Interactive prompts for missing input (via `inquire`)
* Show current DNS configuration
* Track DNS changes (history stack)
* Undo last DNS change
* Reset to original system DNS

---

## 📦 Installation

### Build from source

```bash
git clone <your-repo-url>
cd dnsrs
cargo build --release
```

Binary will be at:

```bash
target/release/dnsrs
```

(Optional)

```bash
sudo cp target/release/dnsrs /usr/local/bin/
```

---

## 🚀 Usage

### Add a new profile

```bash
dnsrs add
```

* Prompts for:

  * profile name
  * DNS IPs

Or:

```bash
dnsrs add google 8.8.8.8 8.8.4.4
```

---

### List profiles

```bash
dnsrs list
```

---

### Set DNS profile

```bash
dnsrs set
```

* Opens interactive selector

Or:

```bash
dnsrs set google
```

---

### Show current DNS

```bash
dnsrs current
```

---

### Undo last change

```bash
dnsrs undo
```

---

### Reset to original DNS

```bash
dnsrs reset
```

---

## 🔐 Permissions

`dnsrs` automatically requests elevated privileges when needed.

You do **not** need to run it with `sudo` manually:

```bash
dnsrs set
```

If required, it will prompt:

```bash
[sudo] password for user:
```

---

## 🧠 How It Works

* Profiles stored in:

  ```
  ~/.config/dnsrs/profiles.json
  ```

* History stored in:

  ```
  ~/.config/dnsrs/history.json
  ```

* DNS changes are applied using system tools (`resolvectl` / fallback methods)

---

## ⚠️ Notes

* On modern Linux systems, `/etc/resolv.conf` is often managed automatically.
* `dnsrs` is designed to work with:

  * `systemd-resolved`
  * (future) NetworkManager (`nmcli`)

---

## 🔧 Planned Improvements

* NetworkManager (`nmcli`) backend
* DNS validation
* Edit/delete profiles
* Per-interface DNS
* TUI mode (ratatui)
* Cross-distro backend detection

---

## 🛠 Dependencies

* [`clap`](https://github.com/clap-rs/clap) – CLI parsing
* [`inquire`](https://github.com/mikaelmello/inquire) – interactive prompts
* `serde` / `serde_json` – config storage


---

## 🔤 Shell Autocompletion

`dnsrs` supports shell autocompletion via `clap_complete`.

---

### 🚀 Generate Completions

Run:

```bash
dnsrs completions <shell>
```

Supported shells:

* bash
* zsh
* fish
* elvish
* powershell

---

## 🐚 Bash

### Temporary (current session)

```bash
source <(dnsrs completions bash)
```

### Persistent

```bash
dnsrs completions bash >> ~/.bashrc
```

Then reload:

```bash
source ~/.bashrc
```

---

## 🐚 Zsh

### Temporary

```bash
source <(dnsrs completions zsh)
```

### Persistent

```bash
dnsrs completions zsh > ~/.zsh/_dnsrs
```

Make sure the directory is in your `$fpath`, then:

```bash
autoload -U compinit
compinit
```

---

## 🐚 Fish

```bash
dnsrs completions fish > ~/.config/fish/completions/dnsrs.fish
```


or

``bash
echo "dnsrs completions fish | source" > ~/.config/fish/config.fish
```

---

## 🐚 PowerShell

```powershell
dnsrs completions powershell | Out-String | Invoke-Expression
```

---

## 🐚 Elvish

```bash
dnsrs completions elvish > ~/.elvish/dnsrs.elv
```

---

## 💡 Tips

* Restart your shell if completions don’t appear
* Ensure `dnsrs` is in your `$PATH`
* For Zsh, verify `$fpath` includes the completions directory

---

## ⚠️ Notes

* Autocompletion covers:

  * commands
  * flags
  * arguments
* Dynamic completion (e.g. profile names for `dnsrs set`) is not included yet

---

## 🔥 Quick Setup (recommended)

### Bash

```bash
dnsrs completions bash >> ~/.bashrc && source ~/.bashrc
```

### Zsh

```bash
dnsrs completions zsh > ~/.zsh/_dnsrs && compinit
```

---

If you want, I can upgrade this to:

* auto-install command (`dnsrs completions install`)
* or dynamic profile name completion (next-level UX)

---

## 📄 License

MIT

---

## 💡 Philosophy

Keep it simple:

* no daemon
* no background services
* just a fast CLI that does one thing well

---
