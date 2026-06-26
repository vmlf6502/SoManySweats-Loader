<h1 align="center">SoManySweats Loader</h1>

<p align="center">A Rust TUI that loads <a href=https://github.com/vmlf6502/SoManySweats>SoManySweats</a> (a Hypixel Bedwars stats overlay) into Lunar Client</p>

<img width="960" height="540" alt="image" src="https://github.com/user-attachments/assets/a4cfeba3-56b2-49a6-811e-dc161212003d"/>

## ⚙️ How It Works
When you launch Lunar Client, it overwrites all of the mods it uses before it begins initializing them to defend against sideloading custom mods. However, I've found that there exists a brief 2-3 second window after Lunar Client checks its resources but before it begins initializing Forge mods in which we can literally just overwrite the ReplayMod file and Lunar will still load it.

## ❔ What Does It Do?
To sum it all up, this program's process looks something like this:
 1. Watch the tail of `~/.lunarclient/profiles/1.8/logs/ichor-boot.log` (the Forge console output for Lunar Client)
 2. When you see the line `LUNARCLIENT_STATUS_PREINIT` printed in that log file, copy the SoManySweats jar to the `~/.lunarclient/offline/multiver` directory, overwriting the ReplayMod file
 3. Done! ✅

All the extra bloat in the codebase includes features such as automatic updates, starting when you computer boots up, and, of course, the TUI (which was made with [ratatui](https://github.com/ratatui/ratatui), a very cool Rust crate for "cooking up terminal user interfaces").

## 📄 Notice
I understand that giving a random GitHub repo with 0 stars code execution on your machine definitely sounds sketchy. If you don't trust the executables provided on the release page, you can clone this repo, review the code, and build it yourself. The code itself is fairly simple, consisting of 3 files averaging ~180 lines each (it's kind of a lot for what it needs to do but hey, I'm working on it). If you don't feel confident in your abilty to evaluate the security of this code, you can always paste it in to ChatGPT or Claude and ask if it does anyting evil. Plus, you can also upload the executables to something like VirusTotal, which will scan them for malware.

Alternative to the SoManySweats Loader, you can simply run a one-line command in the terminal before you launch Lunar Client each time. While this method may be less convenient, it also doesn't involve any sketchy downloads, and it's very easy to verify that the command isn't evil. Use the command corresponding to your OS:
 * 🪟🤢&nbsp;&nbsp;<b>Windows (PowerShell)</b>
      
      ```
      $log = "$env:USERPROFILE\.lunarclient\profiles\1.8\logs\ichor-boot.log"; $pos = (Get-Item $log).Length; while ($true) { $fs = [System.IO.FileStream]::new($log, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite); $fs.Seek($pos, [System.IO.SeekOrigin]::Begin) | Out-Null; $reader = [System.IO.StreamReader]::new($fs); $content = $reader.ReadToEnd(); $reader.Close(); $fs.Close(); if ($content -match "LUNARCLIENT_STATUS_PREINIT") { $src = (Get-Item "$env:USERPROFILE\.lunarclient\offline\multiver\somanysweats\SoManySweats-*.jar").FullName; [System.IO.File]::Copy($src, "$env:USERPROFILE\.lunarclient\offline\multiver\ReplayMod-v1_8-2.6.14.jar", $true); break }; $pos = (Get-Item $log).Length; Start-Sleep -Milliseconds 200 }
      ```
 * 🐧✨&nbsp;&nbsp;<b>Linux</b></summary>

      ```
      tail -F ~/.lunarclient/profiles/1.8/logs/ichor-boot.log | grep --line-buffered -m1 "LUNARCLIENT_STATUS_PREINIT" && cp ~/.lunarclient/offline/multiver/somanysweats/SoManySweats-*.jar ~/.lunarclient/offline/multiver/ReplayMod-v1_8-2.6.14.jar
      ```

## License
This project is licensed under MIT, so feel free to fork it and adapt it to your own projects.
