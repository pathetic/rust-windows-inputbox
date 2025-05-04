# Rust Windows InputBox

A simple customizable Windows input dialog box application written in rust.

## Features

- Clean and simple input dialog box
- Customizable UI layout through a text file
- Real-time layout changes (edit the layout file and reopen the dialog)
- Displays user input in console
- Window always stays on top of other applications

## Usage

1. Build the project with Cargo:

   ```
   cargo build --release
   ```

2. Run the application:

   ```
   cargo run --release
   ```

3. Press Enter in the console to show an input box
4. Enter text and click OK to display it in the console
5. Press Ctrl+C to exit the application

## Customizing the Layout

The application looks for a file named `layout.txt` in the current directory to customize the UI elements. If the file doesn't exist, a default layout file will be created.

You can modify any of these properties in the layout file:

```
# Input Box Layout Configuration

# Window title and message
title=Test InputBox
description=How are you doing today? Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat

# Window dimensions
window_width=400
window_height=250

# Background panel
bg_x=0
bg_y=0
bg_width=0
bg_height=0

# Label/description area
label_x=10
label_y=10
label_width=360
label_height=120

# Edit/input field
edit_x=10
edit_y=140
edit_width=360
edit_height=25

# Buttons
cancel_x=220
cancel_y=170
cancel_width=70
cancel_height=25
ok_x=305
ok_y=170
ok_width=70
ok_height=25
```

Simply edit the values in the file and then press Enter in the application to create a new input box with the updated layout.

## Notes

- All coordinates and sizes are in pixels
- The changes to the layout file are applied when a new input box is created
- The window will always appear on top of other applications
- You can customize both the title and the description text by editing the layout file
