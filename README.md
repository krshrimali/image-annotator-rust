## Templates

Create an annotator app:

**Main page:** (folder selection)

```
-----------------------------
|     SELECT THE FOLDER     |
|   ---------------------   |
|   folder_path_selection   |
|   ---------------------   |
-----------------------------
```

**Image page:**

```
-----------------------------
|                           |
|          IMAGE            |
| ZOOM+    RESET      ZOOM- |
|---------------------------|
|      YES        NO        |
-----------------------------
|          EXPORT           |
-----------------------------
```

## Expectations

**A few important notes:**

* The app should be able to the annotated data for the folder if it existed before.
* Option to export to a CSV file
* Zoom option for an image is "must". Pinch zoom will be great, but just zooming in should be supported. Resetting is also needed.
* The app should ideally work on Mac OS, Windows and Linux OS. If it works on one of them initially, that's also fine.

## Steps

1. Make sure that Iced library has the feature to zoom-in to an image. Figure this out, and then move on.
