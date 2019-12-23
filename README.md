# nasskan
A key remapper for Wayland environments.

- Works without X
- Easy configuration with YAML
- Multi key to multi key mapping
- One-shot modifier
- Reacts to attaching of external keyboards

This software is in beta. There's a possibility for your keyboard to be unresponsive. Use at your own risk.

# Install
```sh
git clone https://github.com/tadosappo/nasskan.git
cd nasskan

cargo build --release
cp target/release/nasskan /usr/bin/
cp nasskan.service /etc/systemd/system/
systemctl enable --now nasskan
```

## Configuration
Nasskan reads `/etc/nasskan/config.yaml`. See [examples](https://github.com/tadosappo/nasskan/blob/master/examples).

```
version: 1
device:
  - if:
      ID_VENDOR_ID: <See below>
      ID_MODEL_ID: <See below>
    then:
      - from:
          key: <a KEY you want to remap>
          with:  # optional
            - <a MODIFIER if you want to remap a key combination>
          without:  # optional
            - <a MODIFIER if you want to disable this rule while certain MODIFIER is pressed>
        to:
          key: <a KEY which will get pressed instead of from.key>
          with:  # optional
            - <a MODIFIER which will get pressed instead of from.with>
        tap:  # optional
          key: <If no other key was pressed while "from" key is pressed, then press this KEY>
```

### if
Nasskan has to know that which remapping rules are for which keyboard. In order to do that, nasskan uses so-called udev device properties. You can check your keyboard's device properties by `udevadm info /dev/input/<your keyboard's device file>`. I recommend that you write ID_VENDOR_ID and ID_MODEL_ID in the configuration but writing other properties should be fine.

You can check your keyboard's device file by `libinput list-devices`.

### KEY
[Possible values are defined here](https://github.com/tadosappo/nasskan/blob/aa33a1e50e28dc5ef1f57212b092fdaa6f7e92cf/src/config.rs#L117).

### MODIFIER
[Possible values are defined here](https://github.com/tadosappo/nasskan/blob/master/src/config.rs#L61).
