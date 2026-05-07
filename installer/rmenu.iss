#define RepoRoot GetEnv("RINSTALLER_REPO_ROOT")
#define AppVersion GetEnv("RINSTALLER_APP_VERSION")
#define DefaultDataRoot GetEnv("RINSTALLER_DATA_ROOT")

[Setup]
AppId={{E62D7EB5-6C41-4C94-93B4-7989717C0F6B}
AppName=rMenu
AppVersion={#AppVersion}
AppPublisher=SynrgStudio
DefaultDirName={autopf}\rMenu
DefaultGroupName=rMenu
DisableProgramGroupPage=yes
OutputDir={#RepoRoot}\dist\installers
OutputBaseFilename=rmenu-setup-v{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
PrivilegesRequired=admin
UninstallDisplayIcon={app}\rmenu.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "startup"; Description: "Start rMenu daemon when Windows starts"; GroupDescription: "Startup:"
Name: "launchdaemon"; Description: "Launch rMenu daemon after install"; GroupDescription: "After install:"

[Dirs]
Name: "{code:GetDataRoot}"
Name: "{code:GetDataRoot}\modules"
Name: "{code:GetDataRoot}\companions"
Name: "{code:GetDataRoot}\config"
Name: "{code:GetDataRoot}\state"

[Files]
Source: "{#RepoRoot}\target\release\rmenu.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\target\release\rmenu-daemon.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\target\release\rmenu-module-host.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\target\release\rmenu-updater.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\config_example.ini"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\README.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\INSTALL.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\MODULES_AUTHORING_GUIDE.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\MODULES_OPERATIONS_GUIDE.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\MANIFEST_SPEC_V1.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\docs\companion-and-rmods-workflow.md"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\docs\rmods-registry.md"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#RepoRoot}\docs\update-workflow.md"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\rMenu"; Filename: "{app}\rmenu.exe"; WorkingDir: "{app}"; IconFilename: "{app}\rmenu.exe"
Name: "{autoprograms}\rMenu Daemon"; Filename: "{app}\rmenu-daemon.exe"; Parameters: "--data-dir ""{code:GetDataRoot}"""; WorkingDir: "{app}"; IconFilename: "{app}\rmenu.exe"
Name: "{autoprograms}\rMenu Stop Daemon"; Filename: "{app}\rmenu-daemon.exe"; Parameters: "--quit"; WorkingDir: "{app}"; IconFilename: "{app}\rmenu.exe"
Name: "{autoprograms}\rMenu Logs"; Filename: "{userappdata}\rmenu"; WorkingDir: "{userappdata}\rmenu"; IconFilename: "{app}\rmenu.exe"

[Registry]
Root: HKCU; Subkey: "Software\SynrgStudio\rMenu"; ValueType: string; ValueName: "DataDir"; ValueData: "{code:GetDataRoot}"
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "rmenu-daemon"; ValueData: """{app}\rmenu-daemon.exe"" --hotkey ""ctrl+shift+space"" --rmenu ""{app}\rmenu.exe"" --data-dir ""{code:GetDataRoot}"""; Flags: uninsdeletevalue; Tasks: startup

[Run]
Filename: "{app}\rmenu-daemon.exe"; Parameters: "--quit"; Flags: runhidden skipifdoesntexist
Filename: "{app}\rmenu-daemon.exe"; Parameters: "--hotkey ""ctrl+shift+space"" --rmenu ""{app}\rmenu.exe"" --data-dir ""{code:GetDataRoot}"""; Description: "Launch rMenu daemon"; Flags: nowait postinstall skipifsilent; Tasks: launchdaemon

[UninstallRun]
Filename: "{app}\rmenu-daemon.exe"; Parameters: "--quit"; Flags: runhidden skipifdoesntexist
Filename: "{cmd}"; Parameters: "/C taskkill /IM rmenu-daemon.exe /F >NUL 2>NUL"; Flags: runhidden
Filename: "{cmd}"; Parameters: "/C taskkill /IM rmenu.exe /F >NUL 2>NUL"; Flags: runhidden

[Code]
var
  DataRootPage: TInputDirWizardPage;

function DefaultDataRoot(): String;
begin
  Result := '{#DefaultDataRoot}';
  if Result = '' then
    Result := 'C:\rMenuData';
end;

function ExistingDataRoot(): String;
begin
  if not RegQueryStringValue(HKCU, 'Software\SynrgStudio\rMenu', 'DataDir', Result) then
    Result := DefaultDataRoot();
end;

function GetDataRoot(Param: String): String;
begin
  if DataRootPage <> nil then
    Result := DataRootPage.Values[0]
  else
    Result := ExistingDataRoot();
end;

procedure InitializeWizard();
begin
  DataRootPage := CreateInputDirPage(
    wpSelectDir,
    'Select rMenu data folder',
    'Choose where rMenu stores modules, companions, config, and state.',
    'This folder is preserved during upgrades and uninstall. Default: C:\rMenuData',
    False,
    ''
  );
  DataRootPage.Add('rMenu data folder:');
  DataRootPage.Values[0] := ExistingDataRoot();
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = DataRootPage.ID then begin
    if Trim(DataRootPage.Values[0]) = '' then begin
      MsgBox('Choose a data folder for rMenu.', mbError, MB_OK);
      Result := False;
    end;
  end;
end;

function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  Exec(ExpandConstant('{cmd}'), '/C taskkill /IM rmenu-daemon.exe /F >NUL 2>NUL', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;
