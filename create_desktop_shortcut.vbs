Set WshShell = CreateObject("WScript.Shell")
Set oShellLink = WshShell.CreateShortcut(WshShell.SpecialFolders("Desktop") & "\Redistricting Dev.lnk")

' Get the current directory (where this script is located)
scriptDir = CreateObject("Scripting.FileSystemObject").GetParentFolderName(WScript.ScriptFullName)

' Set shortcut properties
oShellLink.TargetPath = scriptDir & "\setup_env.bat"
oShellLink.WorkingDirectory = scriptDir
oShellLink.Description = "Congressional Redistricting Development Environment"
oShellLink.IconLocation = "%SystemRoot%\System32\cmd.exe, 0"
oShellLink.WindowStyle = 1  ' Normal window

' Save the shortcut
oShellLink.Save

WScript.Echo "Desktop shortcut created successfully!"
WScript.Echo "Location: " & WshShell.SpecialFolders("Desktop") & "\Redistricting Dev.lnk"
