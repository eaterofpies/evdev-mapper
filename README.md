# evdev-mapper
Maps inputs from multiple input devices to a single virtual input device

## Usage
- List the devices available
    ```
    $ ./evdev-mapper -m devices
    | path                 | physical path                  | name
    | -------------------- | ------------------------------ | ----
    | /dev/input/event8    | usb-0000:03:00.0-7.2/input0    | Microsoft SideWinder Force Feedback 2 Joystick
    | /dev/input/event7    | usb-0000:03:00.0-7.1/input0    | SanmosGroup FR-TEC Raptor Throttle
    ```
- List the properties of each device so you know how the axis and buttons are reported
    ```
    $ ./evdev-mapper -m properties -d /dev/input/event8
    Device: Microsoft SideWinder Force Feedback 2 Joystick
    Keys:
        KeyCode(BTN_TOP)
        KeyCode(BTN_TRIGGER)
        KeyCode(BTN_BASE)
        KeyCode(BTN_THUMB2)
        KeyCode(BTN_TOP2)
        KeyCode(BTN_BASE2)
        KeyCode(BTN_THUMB)
        KeyCode(BTN_PINKIE)
        KeyCode(BTN_DEAD)
    Absolute axis:
        AbsoluteAxisType(ABS_X): AbsInfo { value: 0, min: -512, max: 511, fuzz: 3, flat: 63, resolution: 0 }
        AbsoluteAxisType(ABS_HAT0Y): AbsInfo { value: 0, min: -1, max: 1, fuzz: 0, flat: 0, resolution: 0 }
        AbsoluteAxisType(ABS_Y): AbsInfo { value: 45, min: -512, max: 511, fuzz: 3, flat: 63, resolution: 0 }
        AbsoluteAxisType(ABS_RZ): AbsInfo { value: 0, min: -32, max: 31, fuzz: 0, flat: 3, resolution: 57 }
        AbsoluteAxisType(ABS_THROTTLE): AbsInfo { value: 100, min: 0, max: 127, fuzz: 0, flat: 7, resolution: 0 }
        AbsoluteAxisType(ABS_HAT0X): AbsInfo { value: 0, min: -1, max: 1, fuzz: 0, flat: 0, resolution: 0 }
    ```
- Load `device.conf` to map input devices to a virtual input device
    ```
    $ ./evdev-mapper
    ```

## Configuration
In the configuration file you can specify one or more inputs devices and how to represent events from those devices on a virtual input device
```
- path: <path to input device 1>
  mappings:
    - input_event: <input 1>
      output_event: <output 1>
    - input_event: <input 2>
      output_event: <output 2>
    ...
- path: <path to input device 2>
  mappings:
    - input_event: <input 3>
      output_event: <output 3>
    ...
```

You can map
- A button to a button
    ```
    - input_event: BTN_0
      output_event: BTN_1
    ```
- An absolute axis to an absolute axis
    ```
    - input_event: ABS_X
      output_event: ABS_Y
    ```
- A range of an absolute axis to a button
    ```
    # map a hat switch axis to 2 buttons
    - input_event: ABS_HAT0X
      output_event:
          - min: 1
            max: 1
            key: BTN_0
          - min: -1
            max: -1
            key: BTN_1
    ```

Note: Mapping inputs to BTN_LEFT and BTN_RIGHT causes the device to be detected as a mouse which may / may not be what you want.
