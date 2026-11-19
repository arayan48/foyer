; Foyer Client : après installation ou mise à jour, Windows recharge les icônes (raccourcis à jour)

!macro NSIS_HOOK_POSTINSTALL
  nsExec::Exec 'ie4uinit.exe -show'
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend
