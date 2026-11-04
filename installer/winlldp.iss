#define MyAppName "WinLLDP"
#define MyAppVersion "1.2.0"
#define MyAppPublisher "Dominik Tröster, Verbandsgemeinde Otterbach-Otterberg"

[Setup]
AppId=WinLLDP.DominikTroester
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL=https://github.com/derdigitalaffine/winlldp
AppSupportURL=https://github.com/derdigitalaffine/winlldp/issues
AppUpdatesURL=https://github.com/derdigitalaffine/winlldp/releases
DefaultDirName={localappdata}\Programs\WinLLDP
DefaultGroupName=WinLLDP
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
UninstallDisplayIcon={app}\WinLLDP.exe
Compression=lzma2
SolidCompression=yes
OutputDir=..\build
OutputBaseFilename=WinLLDP-1.2.0-Windows-x64-Setup
SetupIconFile=..\assets\winlldp.ico
LicenseFile=..\LICENSE
WizardStyle=modern
VersionInfoVersion=1.2.0.0
VersionInfoProductVersion=1.2.0.0
VersionInfoDescription=WinLLDP Windows x64 installer

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "dutch"; MessagesFile: "compiler:Languages\Dutch.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\winlldp.exe"; DestDir: "{app}"; DestName: "WinLLDP.exe"; Flags: ignoreversion
Source: "..\target\x86_64-pc-windows-msvc\release\winlldp-cli.exe"; DestDir: "{app}"; DestName: "winlldp-cli.exe"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\Cargo.lock"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\WinLLDP"; Filename: "{app}\WinLLDP.exe"
Name: "{group}\WinLLDP CLI"; Filename: "{cmd}"; Parameters: "/K ""{app}\winlldp-cli.exe"" --help"; WorkingDir: "{app}"
Name: "{autodesktop}\WinLLDP"; Filename: "{app}\WinLLDP.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\WinLLDP.exe"; Description: "{cm:LaunchProgram,WinLLDP}"; Flags: nowait postinstall skipifsilent
