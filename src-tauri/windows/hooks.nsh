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
