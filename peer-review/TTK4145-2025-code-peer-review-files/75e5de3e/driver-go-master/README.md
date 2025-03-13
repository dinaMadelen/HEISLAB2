# Hei, og velkommen til oss.

<br></br>

## Introduksjon

_Vi har laget et master-slave system, der vi har 3. følgende moduler: **primaryProcess**, **backupProcess**, og **slaveProcess**. Hver av disse modulene skal være en fungerende single elevator. Kort formulert fungerer modulene som følger:_
1. Slaven sender hallcalls med verdensbilde til master.
2. Master tar imot hallcalls og verdensbilder fra alle heiser.
3. Master sender alle hallcalls til backup.
4. Når backup acknowledger denne meldingen kjører master en algoritme for å så distribuere følgende hallcalls.
5. Disse blir så sendt tilbake til heisene. 

## Slave
_Slaven sin oppgave er å ha en fungerende single elevator, når det skjer et knappetrykk sender den knappe-trykket videre til master. Den vil også lytte til master for hallcalls._

SlaveProcess()
- Initialiserer heisen og dens tilstand.
- Starter gorutiner for å overvåke knapper, etasjer, obstruksjon, stoppknapp og heartbeat.
- Håndterer etasjeankomster, knappetrykk og hindringer.
- Sender hall call-meldinger til master ved knappetrykk.
- Stopper motoren ved obstruksjon.
- Håndterer dørtimeout.
- sender heartbeat-meldinger.

## Backup

_Backup sin oppgave er å ta imot kopi av master, og ta over som ny master dersom nåværende master dør. Den vil også fungere som en "vanlig slave", det vil si at den sender knappetrykk etc til master._


BackupProcess()
- Initialiserer heisens tilstand og sensorer.
- Starter gorutiner for knappe-, etasje-, obstruksjons- og stoppknappovervåkning.
- Sender og lytter til heartbeat-meldinger.
- Tar over som primærprosess hvis master dør.

BackupProcessListenToMessages(ch chan<- bool)
- Lytter etter meldinger på UDP.
- Overvåker heartbeat fra master.
- Hvis flere heartbeat-meldinger mangler, signaliserer den at backup skal ta over.

BackupProcessAcknowledgeMaster(checkpoint int)
- Sender en bekreftelse tilbake til master med gjeldende checkpoint-status.

## Master

_Tar imot hallcalls fra alle heiser. Den sender kopi til backup, og først når den får acknowledgement vil den kjøre en algoritme for å tildele hallcalls på en sofistikert måte. Den vil så sende hallcalls videre til slaves. Først nå kan lysene slåes på._


PrimaryProcess()

- Initialiserer heisens tilstand og sensorer.
- Starter gorutiner for knappe-, etasje-, obstruksjons- og stoppknappovervåkning.
- Sender heartbeat-meldinger og lytter etter meldinger.
- Behandler heisens knappe- og etasjesignaler og distribuerer meldinger.

PrimaryProcessMasterListener(ch chan string)
- Lytter etter UDP-meldinger og sender dem videre på channelen ch.

PrimaryProcessMasterSendHeartbeat()
- Sender regelmessige heartbeat-meldinger over UDP for å signalisere at master er aktiv.

<br></br>
## Andre filer
_Våre tre moduler bruker andre mapper og filer for å fungere._

### Main

_Initialiserer en heis som backup som kobler seg på en bestemt IP. Skal utvides til å kunne starte en heis som slave også._

### Request / Elevator / FSM / Timer / Elevio

_Disse mappene sørger for at single elevator fungerer tilfredstillenede. Her har vi tatt inspirasjon fra https://github.com/ttk4145._  

### Utilities

_Dette er en mappe som inneholder hjelpe-funksjoner som kan bli brukt av alle modulene._

- jsonstring.go er en hjelpefil for å håndtere alle meldinger som sendes over nettverket. 
- Tag representerer de ulike meldingstypene som sendes over nettverket. 
- Acknowledgement brukes av backup for å bekrefte til master om mottat melding
- Button Press indikerer at en knapp har blitt trykket 
- HeartbeatX er statusmeldinger fra slave, backup eller master, som inneholder hele verdensbilde til de ulike . 
- ElevatorState representerer tilstanden til en heis 
- SetJsonString tar inn en Tag 
