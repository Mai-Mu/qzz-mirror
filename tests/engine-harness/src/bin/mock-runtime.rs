use std::{env, fs, io::{Read,Write}, path::PathBuf, thread, time::Duration};
fn main() {
 let state = PathBuf::from(env::var("QZZ_MOCK_STATE").expect("isolated test state"));
 let name = env::current_exe().unwrap().file_stem().unwrap().to_string_lossy().to_string();
 let args = env::args().skip(1).collect::<Vec<_>>();
 let a = args.iter().map(String::as_str).collect::<Vec<_>>();
 if name == "scrcpy" {
  if a.contains(&"--version") { println!("scrcpy 5.0.1 <mock>"); return; }
  let serial = a.windows(2).find(|p| p[0] == "--serial").unwrap()[1];
  fs::write(state.join("scrcpy-args.json"), serde_json::to_vec(&args).unwrap()).unwrap();
  let mut starts = fs::OpenOptions::new().create(true).append(true).open(state.join("scrcpy-starts.log")).unwrap(); writeln!(starts,"{serial}").unwrap();
  while state.join("delay-start").exists() { thread::sleep(Duration::from_millis(30)); }
  println!("INFO: Texture (D3D11VA): 480x800");
  loop {
   if state.join("close-window").exists() { println!("DEBUG: User requested to quit"); return; }
   let marker = if serial.contains(':') { "wifi-online" } else { "usb-online" };
   if !state.join(marker).exists() { eprintln!("WARN: Device disconnected"); std::process::exit(2); }
   thread::sleep(Duration::from_millis(20));
  }
 }
 let mut calls = fs::OpenOptions::new().create(true).append(true).open(state.join("adb-calls.log")).unwrap(); writeln!(calls,"{}", a.join(" ")).unwrap();
 match a.as_slice() {
  ["devices", "-l"] => {
   println!("List of devices attached");
   if state.join("usb-online").exists() { println!("USB-123 device usb:1-2 model:Test_Android transport_id:1"); }
   if state.join("wifi-online").exists() { println!("192.168.1.22:5555 device model:Test_Android transport_id:2"); }
  }
  ["-s", serial, "shell", "getprop"] => {
   if state.join("delay-metadata").exists() {
    fs::write(state.join("metadata-waiting"), "").unwrap();
    while state.join("delay-metadata").exists() { thread::sleep(Duration::from_millis(20)); }
   }
   let identity = if serial.contains(':') && state.join("identity-missing").exists() { "" } else if serial.contains(':') && state.join("identity-b").exists() { "HARDWARE-OTHER" } else { "HARDWARE-123" }; println!("[ro.serialno]: [{identity}]\n[ro.product.model]: [Test Android]\n[ro.build.version.release]: [9]\n[ro.build.version.sdk]: [28]");
  },
  ["-s", _, "shell", "getprop", "ro.product.model"] => println!("Test Android"),
  ["-s", _, "shell", "ip", "-4", "route"] => println!("192.168.1.0/24 dev wlan0 proto kernel scope link src 192.168.1.22"),
  ["-s", _, "tcpip", "5555"] => println!("restarting in TCP mode port: 5555"),
  ["connect", address] => { fs::write(state.join("wifi-online"), "").unwrap(); println!("connected to {address}"); },
  ["pair", address] => { let mut code=String::new(); std::io::stdin().read_to_string(&mut code).unwrap(); assert_eq!(code.trim(),"123456"); println!("Successfully paired to {address}"); },
  ["mdns", "services"] => { println!("List of discovered mdns services"); if state.join("mdns-connect").exists() { println!("adb-DEVICE _adb-tls-connect._tcp 192.168.1.22:44444"); } },
  _ => { eprintln!("unexpected mock args: {args:?}"); std::process::exit(1); }
 }
}


