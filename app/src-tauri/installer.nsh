; Foyer Serveur : l'agent démarre avec Windows ; l'interface est au choix.

!macro NSIS_HOOK_PREINSTALL
  ; mise à jour : arrêt propre des jeux (sauvegarde des mondes), puis libère foyer-agent.exe
  nsExec::Exec `powershell -NoProfile -Command "try { $$t = (Get-Content C:\Foyer\token -Raw).Trim(); Invoke-RestMethod -Method Post -Uri http://127.0.0.1:7878/api/shutdown -Headers @{Authorization=('Bearer ' + $$t)} -TimeoutSec 120 } catch {}"`
  nsExec::Exec 'schtasks /End /TN "Foyer Agent"'
  nsExec::Exec 'taskkill /F /T /IM foyer-agent.exe'
  Sleep 1000
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; tâche déjà là = mise à jour : on redémarre l'agent sans reposer de question
  nsExec::Exec 'schtasks /Query /TN "Foyer Agent"'
  Pop $0
  ${If} $0 == 0
    nsExec::Exec 'schtasks /Run /TN "Foyer Agent"'
    Goto foyer_done
  ${EndIf}
  nsExec::Exec 'schtasks /Create /TN "Foyer Agent" /TR "\"$INSTDIR\foyer-agent.exe\"" /SC ONSTART /RU SYSTEM /RL HIGHEST /F'
  nsExec::Exec 'netsh advfirewall firewall delete rule name="Foyer Agent"'
  nsExec::Exec 'netsh advfirewall firewall add rule name="Foyer Agent" dir=in action=allow protocol=TCP localport=7878 remoteip=localsubnet'
  nsExec::Exec 'schtasks /Run /TN "Foyer Agent"'
  MessageBox MB_YESNO|MB_ICONQUESTION "Installer aussi l'interface sur ce serveur ?$\r$\n$\r$\nOui : une fenêtre pour gérer les jeux directement ici.$\r$\nNon : le serveur tourne sans fenêtre, tu le gères depuis un PC avec Foyer Client." /SD IDYES IDYES foyer_done
    Delete "$DESKTOP\Foyer Serveur.lnk"
    Delete "$SMPROGRAMS\Foyer Serveur.lnk"
    RMDir /r "$SMPROGRAMS\Foyer Serveur"
  foyer_done:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'schtasks /End /TN "Foyer Agent"'
  nsExec::Exec 'schtasks /Delete /TN "Foyer Agent" /F'
  nsExec::Exec 'taskkill /F /T /IM foyer-agent.exe'
  nsExec::Exec 'netsh advfirewall firewall delete rule name="Foyer Agent"'
!macroend
