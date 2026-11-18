; Foyer Serveur : l'agent démarre avec Windows ; l'interface est au choix.

!macro NSIS_HOOK_PREINSTALL
  ; mise à jour : arrêt propre des jeux (sauvegarde des mondes), puis libère foyer-agent.exe
  nsExec::Exec `powershell -NoProfile -Command "try { $$t = (Get-Content C:\Foyer\token -Raw).Trim(); $$h = @{Authorization=('Bearer ' + $$t)}; [Net.ServicePointManager]::SecurityProtocol = 'Tls12'; [Net.ServicePointManager]::ServerCertificateValidationCallback = {$$true}; try { Invoke-RestMethod -Method Post -Uri https://127.0.0.1:7878/api/shutdown -Headers $$h -TimeoutSec 120 } catch { Invoke-RestMethod -Method Post -Uri http://127.0.0.1:7878/api/shutdown -Headers $$h -TimeoutSec 120 } } catch {}"`
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
  ; arrêt propre des jeux (sauvegarde des mondes) avant de tout couper
  nsExec::Exec `powershell -NoProfile -Command "try { $$t = (Get-Content C:\Foyer\token -Raw).Trim(); $$h = @{Authorization=('Bearer ' + $$t)}; [Net.ServicePointManager]::SecurityProtocol = 'Tls12'; [Net.ServicePointManager]::ServerCertificateValidationCallback = {$$true}; Invoke-RestMethod -Method Post -Uri https://127.0.0.1:7878/api/shutdown -Headers $$h -TimeoutSec 120 } catch {}"`
  nsExec::Exec 'schtasks /End /TN "Foyer Agent"'
  nsExec::Exec 'schtasks /Delete /TN "Foyer Agent" /F'
  nsExec::Exec 'taskkill /F /T /IM foyer-agent.exe'
  Sleep 1000
  ; règles pare-feu : « Foyer Agent » et celles des jeux (« Foyer <id> TCP <port> »)
  nsExec::Exec `powershell -NoProfile -Command "Get-NetFirewallRule -DisplayName 'Foyer *' -ErrorAction SilentlyContinue | Remove-NetFirewallRule"`
  ; données : seulement sur demande explicite, jamais en mode silencieux
  MessageBox MB_YESNO|MB_ICONEXCLAMATION|MB_DEFBUTTON2 "Supprimer aussi tous les serveurs de jeu et leurs données ?$\r$\n$\r$\nMondes, sauvegardes, réglages et jeton (C:\Foyer) seront effacés définitivement.$\r$\nNon : ils restent sur le disque pour une réinstallation." /SD IDNO IDNO foyer_keep
    RMDir /r "C:\Foyer"
  foyer_keep:
!macroend
