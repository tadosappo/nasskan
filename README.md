# nasskan
A key remapper for Wayland environments.

- Works without X
- Easy configuration with YAML
- From key combination to key combination mapping
- One-shot modifier
- Reacts to attach of external keyboards

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
          key: <If no other key was pressed while "from" key is pressed, then this KEY gets pressed>
```

### if
Nasskan has to know which keyboard the remapping rules are for. In order to do so, nasskan uses udev device properties such as ID_VENDOR or ID_MODEL. You can check your keyboard's device properties by `udevadm info /dev/input/<your keyboard's device file>`. You can check your keyboard's device file path by `libinput list-devices`. I recommend that you write your keyboard's ID_VENDOR_ID and ID_MODEL_ID in `if` section. but writing other properties should be fine.

### KEY
[Possible values are defined here](https://github.com/tadosappo/nasskan/blob/4f064d3c7292e4d0d3ef3e6bd7649f3d7ad6c65c/src/config.rs#L124).

### MODIFIER
[Possible values are defined here](https://github.com/tadosappo/nasskan/blob/4f064d3c7292e4d0d3ef3e6bd7649f3d7ad6c65c/src/config.rs#L61).
