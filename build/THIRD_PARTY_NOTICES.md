# Third-party notices

WinLLDP's own code and artwork are distributed under the MIT License (LICENSE).

The application includes Rust libraries from the dependency versions recorded in Cargo.lock. Key dependencies include winptap (MIT OR Apache-2.0; native Windows Packet Monitor binding), winptap-core, winptap-pktmon and winptap-native (MIT OR Apache-2.0), egui/eframe (MIT OR Apache-2.0), image (MIT OR Apache-2.0), serde/serde_json (MIT OR Apache-2.0), chrono (MIT OR Apache-2.0), libloading (ISC), ipconfig (MIT OR Apache-2.0), tray-icon/muda (MIT OR Apache-2.0), rfd (MIT), windows-sys (MIT OR Apache-2.0), winreg (MIT), and bundled egui fonts (Ubuntu Font Licence / SIL Open Font License, as specified in their source distribution). Full dependency license texts are supplied with release artifacts in THIRD_PARTY_LICENSES.txt.

Npcap is an optional, separately installed dependency. Npcap is not included in this repository's binaries or installer and is not covered by WinLLDP's MIT License. Obtain it and review its installation/use/redistribution terms at https://npcap.com/#download and https://npcap.com/guide/ . WinLLDP does not install a service; Npcap itself uses a kernel driver.
