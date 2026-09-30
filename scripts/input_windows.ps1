param(
    [Parameter(Mandatory=$true)][int]$GamePid,
    [Parameter(Mandatory=$true)][ValidateSet('F2')][string]$Key
)
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class SalimonInput {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
}
'@
$game = Get-Process -Id $GamePid
if ($game.MainWindowHandle -eq 0) { throw 'Game has no visible main window' }
if (-not [SalimonInput]::SetForegroundWindow($game.MainWindowHandle)) { throw 'Cannot focus game window' }
if ([SalimonInput]::GetForegroundWindow() -ne $game.MainWindowHandle) { throw 'Game did not receive focus' }
$code = [byte]0x71
[SalimonInput]::keybd_event($code, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
[SalimonInput]::keybd_event($code, 0, 2, [UIntPtr]::Zero)
