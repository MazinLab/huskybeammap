# HuskyBeammap

This repository has two main parts, huskybeammap-app which runs on a the beammapper phone (or potentially on the native and the web), and huskybeammap-proxy which the phone connects to to recieve commands.

# HuskyBeammap-App

This can be built with cargo `quad-apk` assuming you have the android NDK/SDK installed. Otherwise it will run just fine on your laptop without modifications. You can run it with `cargo run --release huskybeammap-app`

By default it will automatically attempt to connect to the address that the phone assigns the first connected device in usb tethering mode, if this fails it will give you the option to connect to another address.

# HuskyBeammap-Proxy

This is a bridge that opens a websocket server on port 9001 and port 9002. Beammapper phones will connect on port 9001 to recieve commands, and it will relay any commands sent by client scripts connecting to port 9002.

An example script is contained in `demo.py` and (assuming you have the websockets library installed) can be run with `python3 demo.py localhost 9002`
