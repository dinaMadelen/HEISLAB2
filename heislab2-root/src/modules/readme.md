# modules/
Her bor alle moduler i full harmoni!

## Hvordan legge til nye moduler

### Hvordan legge til enkeltfils-moduler:
1. Lagre fil her: modules/\<filnavn>
2. Deklarer modul i modules/mod.rs med: pub mod \<filnavn>; //(filnavn uten ".rs")

### Hvordan legge til multifils-moduler:
1. Lag ny mappe i denne mappen (/modules/\<mappenavn>)
2. Deklarer mappenavn i modules/mod.rs med pub mod \<mappenavn>;
3. Lag en ny mod.rs fil i den nye mappen (modules/\<mappenavn>/mod.rs)
4. Deklarer alle sub-moduler i modules/\<mappenavn>/mod.rs med: pub mod \<filnavn>; //(filnavn uten ".rs")
