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

  ; Clear any leftover per-user install/data in %LOCALAPPDATA%\TalkU so a fresh
  ; install doesn't conflict with stale per-user files/registry.
  ; RMDir /r removes recursively.
  ${If} ${FileExists} "$LOCALAPPDATA\TalkU"
    RMDir /r "$LOCALAPPDATA\TalkU"
  ${EndIf}

  ; Stop and remove any previously-installed TalkUCLI service. A running old
  ; service would lock talku-cli.exe and prevent this installer from
  ; overwriting it, and re-registering below needs a clean slate anyway.
  ; (These are no-ops/fail silently when no service exists yet.)
  nsExec::ExecToLog 'sc stop TalkUCLI'
  nsExec::ExecToLog 'sc delete TalkUCLI'

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
