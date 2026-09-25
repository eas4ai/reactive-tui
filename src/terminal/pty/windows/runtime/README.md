# Pinned Microsoft ConPTY runtime

These unmodified files come from Microsoft.Windows.Console.ConPTY 1.24.260710001,
published by Microsoft.Terminal under the MIT license. The shim DLL and console
host must come from the same package; they share a private protocol.

Package URL:
https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/1.24.260710001/microsoft.windows.console.conpty.1.24.260710001.nupkg

Package SHA-256:
`175640566a3b59c4b132070ee96c2c77e5ab7edd2e92732a5eb3610bbf63d90e`

For each architecture (`x86`, `x64`, `arm64`), `conpty.dll` comes from
`runtimes/win-ARCH/native/conpty.dll`; `OpenConsole.exe` comes from
`build/native/runtimes/ARCH/OpenConsole.exe`. `manifest.json` records each file's
SHA-256 and is compiled into the runtime loader. LICENSE retains Microsoft's
copyright and MIT terms from https://github.com/microsoft/terminal/blob/main/LICENSE.

Use `scripts/install-conpty-runtime.py` to package the application architecture's
DLL and every native console host. No network access occurs during installation
or application startup. Update package, manifest, loader compatibility and native
verification together. Never replace just one member of the pair.
