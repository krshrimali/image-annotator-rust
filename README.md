## Description

**Welcome page - select your folder**

<img width="1022" alt="image" src="https://user-images.githubusercontent.com/19997320/212478003-65eeba74-f894-4609-8fcc-b95ec88b8db7.png">

- Any sub-folders present in the selected folder will be ignored.
- File validation is done while traversing through the folder, to save time.

**Verify annotation**

<img width="1022" alt="image" src="https://user-images.githubusercontent.com/19997320/212478037-3126f00d-571f-4b6e-ba23-bac27f7f27c0.png">

- Option to mark as correct/incorrect or reset selection.
- Click `Export` to export the results to a JSON file. (`output.json` in the folder where you started the app from)
- Mark as Incorrect will have an option to add comments (optional)

**Add comments (optional)**

<img width="1022" alt="image" src="https://user-images.githubusercontent.com/19997320/212478086-fd284f0e-5bb0-44ef-84a0-058b55ee8671.png">

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
