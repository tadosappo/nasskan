# nasskan
A key remapper for Wayland.

- ✔ Easy configuration in YAML
- ✔ One-shot modifier
- ✔ Map key combinations to key combinations
- ✔ Monitors attaching/detatching of external keyboards

This software is in alpha state. Your keyboard will possibly be unresponsive. Use at your own risk.

## Install
TODO: Open Build System

## Usage
```sh
systemctl enable --now nasskan
```

## Configuration
Nasskan reads `/etc/nasskan/config.yaml`. See [examples](https://github.com/tadosappo/nasskan/blob/master/examples).

```
version: 1
devices:
  - vendor_id: 
    product_id: 
    rules:
      - from:
          key: <a KEY you want to remap>
          with:  # optional
            - <a MODIFIER if you want to remap a key combination>
          without:  # optional
            - <a MODIFIER if you want to disable this rule while certain MODIFIER is pressed>
        to:
          key: <a KEY which will get pressed instead of the "from" key>
          with:  # optional
            - <a MODIFIER which will get pressed in addition to modifiers pressed actually>
          without:  # optional
            - <a MODIFIER which will get released before the key get pressed>
        tap:  # optional
          key: <If no other key was pressed while "from" key is pressed, then press this KEY>
```

## Terms
### KEY
[Possible values are defined here as constants](https://github.com/torvalds/linux/blob/b5625db9d23e58a573eb10a7f6d0c2ae060bc0e8/include/uapi/linux/input-event-codes.h#L77). You can write either key code number or key code name without `KEY_`.

### MODIFIER
Possible values are `SHIFT`, `CTRL`, `ALT`, `META` (for Super key, Windows key, Command key, or whatever).
