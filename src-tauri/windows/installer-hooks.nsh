; Stop the bundled agent-platformd before its exe is overwritten or removed.
; Force-closing Portal (which the installer does) skips the app's own cleanup and
; leaves the sidecar running, and the install aborts with "Can't write".
; So close Portal first, then the sidecar.
; ponytail: matches by process name, so it also stops a dev build's sidecar.
!macro STOP_PORTAL_AND_SIDECAR
  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  nsis_tauri_utils::KillProcessCurrentUser "agent-platformd.exe"
  Pop $R0
  Sleep 1000 ; let Windows release the file handle
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro STOP_PORTAL_AND_SIDECAR
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro STOP_PORTAL_AND_SIDECAR
!macroend
