## Key Features

**Note**: I'm actively working on this app's first release. Please hop on to the [issues page](https://github.com/krshrimali/image-annotator-rust-app/issues) if you would like to contribute.

1. Select folder with images to mark annotated images.
2. Zoom (like pinch zoom) and pan images in the view.
3. Options available: Mark as Correct, Mark as Incorrect, Reset Selection.
4. Export as a JSON file.
5. The JSON file can retain previously annotated folders.
6. See the info (track current image path, folder path, total files etc.) in the window itself.
7. Invalid files are ignored, and a proper text is shown instead of the image viewer.
8. Any sub-directories in the selected folder are ignored.
9. Image file sizes are retained, and the app is scrollable + resizable.
10. Binaries are available for Windows, OSX and Linux, [here](https://github.com/krshrimali/validate-image-annotations-rust/tree/main/binaries).
11. Built 100% with Rust, GUI built using [Iced library](https://github.com/iced-rs/iced/)

## Description and Demo

**Theme Switching**

The app saves you some eye strain with dark and light theme switching option.

![Screenshot from 2023-01-29 12-54-20](https://user-images.githubusercontent.com/19997320/215311668-bc935e22-fcc6-4882-bf7e-21d48b6173d0.png)

**Welcome page - select your folder**

<img width="1022" alt="image" src="https://user-images.githubusercontent.com/19997320/212478003-65eeba74-f894-4609-8fcc-b95ec88b8db7.png">

- Any sub-folders present in the selected folder will be ignored.
- File validation is done while traversing through the folder, to save time.

**Verify annotation**

<img width="1022" alt="image" src="https://user-images.githubusercontent.com/19997320/212478037-3126f00d-571f-4b6e-ba23-bac27f7f27c0.png">

- Option to mark as correct/incorrect or reset selection.
- Click `Export` to export the results to a JSON file. (`output.json` in the folder where you started the app from)
- Mark as Incorrect will have an option to add comments (optional)

**Invalid file**

![invalid_file](https://user-images.githubusercontent.com/19997320/215312196-bdc46516-2dd8-4fb2-8b00-416e9fbd47ed.png)

- In case a file is invalid, or the image couldn't be loaded, a message will appear and a user can see the file path in the info below to the text.

**Add comments (optional)**

![output_annotation](https://user-images.githubusercontent.com/19997320/215312198-0a089cc0-1a18-4727-b433-a9ad0a9b91c3.jpeg)

## Output

A sample output is given [here](https://github.com/krshrimali/image-annotator-rust-app/blob/main/output.json)

```json
{
  "image_to_properties_map": {
    "/Users/krshrimali/Documents/krshrimali/projects/image-annotator-rust-app/sample_folder": [
      {
        "index": 0,
        "image_path": "/Users/krshrimali/Documents/krshrimali/projects/image-annotator-rust-app/sample_folder/sample.webp",
        "annotation": true,
        "comments": ""
      },
    ],
    "/Users/krshrimali/Documents/Photos": [
      {
        "index": 0,
        "image_path": "/Users/krshrimali/Documents/Photos/Kush.png",
        "annotation": true,
        "comments": ""
      },
    ]
  }
}
```

## Build from source

If you are on Linux, following libraries are required:

1. Rust toolchain, see: [this](https://doc.rust-lang.org/cargo/getting-started/installation.html) for instructions.
2. `cmake`, `pkg-config`, `fontconfig`.
3. [OpenSSL 3.0](https://openssl.org/)
4. `libgtk-3-dev` (GTK 3.0 dev libraries)

Once done, follow the instructions below:

```shell
# Build porject
cargo build
# Build binary for release
cargo build --release
# Run the app
cargo run
```
