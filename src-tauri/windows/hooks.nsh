!macro NSIS_HOOK_PREINSTALL

  ; If we are not running elevated, relaunch the installer elevated so the
  ; service-registration steps below can run as admin (shows the UAC prompt).
  ; The elevated copy of the installer IS admin, so it skips this block and
  ; proceeds with the normal install.
  UserInfo::GetAccountType
  Pop $0
  StrCmp $0 "Admin" +5 ; admin -> skip elevation logic
  MessageBox MB_ICONINFORMATION|MB_YESNO "TalkU needs administrator rights to install (it registers a system service). Continue?" IDYES +2
  Abort
  ExecShell "runas" '"$EXEPATH"'
  Quit

  ; 1. Stop and remove any previously-installed TalkUCLI service FIRST so
  ;    talku-cli.exe releases all file locks before we delete or overwrite files.
  ;    `net stop` waits synchronously for the service to stop (unlike `sc stop`),
  ;    and `taskkill` ensures any lingering process is terminated immediately.
  nsExec::ExecToLog 'net stop TalkUCLI'
  nsExec::ExecToLog 'taskkill /F /IM talku-cli.exe /T'
  Sleep 500
  nsExec::ExecToLog 'sc delete TalkUCLI'

  ; 2. Now that the service process is closed and files are unlocked, clear any
  ;    leftover per-user install/data in %LOCALAPPDATA%\TalkU.
  ${If} ${FileExists} "$LOCALAPPDATA\TalkU"
    RMDir /r "$LOCALAPPDATA\TalkU"
  ${EndIf}

!macroend

!macro NSIS_HOOK_POSTINSTALL

  ; Register the TalkUCLI daemon as a per-machine Windows service running as
  ; SYSTEM (LocalSystem). The service starts AUTOMATICALLY with the system
  ; (`start= auto`), so the daemon's named pipe is available without the app
  ; needing to start it. talku-cli.exe's service entry point is handled by the
  ; `windows-service` crate, so this binary is a proper WIN32_OWN_PROCESS
  ; service (the default type for `sc create`).
  nsExec::ExecToLog 'sc create TalkUCLI binPath= "\"$INSTDIR\talku-cli.exe\"" start= auto obj= "LocalSystem" DisplayName= "TalkU CLI Service"'

  ; Start the service now so it is running immediately after install.
  nsExec::ExecToLog 'sc start TalkUCLI'

  ; Friendly description shown in services.msc.
  nsExec::ExecToLog 'sc description TalkUCLI "TalkU elevated daemon service run by the TalkU desktop app"'

!macroend

!macro NSIS_HOOK_PREUNINSTALL

  ; Stop and unregister the TalkUCLI service before the uninstaller removes files
  ; from $INSTDIR so talku-cli.exe and wireguard-amd64.dll are not locked.
  nsExec::ExecToLog 'net stop TalkUCLI'
  nsExec::ExecToLog 'taskkill /F /IM talku-cli.exe /T'
  Sleep 500
  nsExec::ExecToLog 'sc delete TalkUCLI'

!macroend
