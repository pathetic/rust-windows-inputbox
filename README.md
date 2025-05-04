# Input Box

A simple customizable Windows input dialog box application.

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
# Window title and message
title=Input Box
description=Enter your text:

# Window dimensions
window_width=400
window_height=200

# Background panel
bg_x=0
bg_y=30
bg_width=400
bg_height=110

# Label/description area
label_x=10
label_y=45
label_width=380
label_height=40

# Edit/input field
edit_x=20
edit_y=95
edit_width=360
edit_height=25

# Buttons
cancel_x=210
cancel_y=150
cancel_width=80
cancel_height=25
ok_x=300
ok_y=150
ok_width=80
ok_height=25
```

Simply edit the values in the file and then press Enter in the application to create a new input box with the updated layout.

## Notes

- All coordinates and sizes are in pixels
- The changes to the layout file are applied when a new input box is created
- The window will always appear on top of other applications
- You can customize both the title and the description text by editing the layout file
