!macro NSIS_HOOK_POSTINSTALL

  ; Register the TalkUCLI daemon as a per-machine Windows service running as
  ; SYSTEM (LocalSystem). The service is on-demand (`start= demand`): the TalkU
  ; app starts it with `sc start TalkUCLI` (see src-tauri/src/lib.rs), which
  ; causes NO UAC prompt because the service principal already runs elevated as
  ; SYSTEM. talku-cli.exe's service entry point is handled by the
  ; `windows-service` crate, so this binary is a proper WIN32_OWN_PROCESS
  ; service (the default type for `sc create`).
  nsExec::ExecToLog 'sc create TalkUCLI binPath= "\"$INSTDIR\talku-cli.exe\"" start= demand obj= "LocalSystem" DisplayName= "TalkU CLI Service"'

  ; Friendly description shown in services.msc.
  nsExec::ExecToLog 'sc description TalkUCLI "TalkU elevated daemon service run by the TalkU desktop app"'

!macroend
