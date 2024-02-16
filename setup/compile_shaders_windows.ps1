# Get all files in the .\shaders folder
$files = Get-ChildItem -Path .\shaders

# Iterate over each file
foreach ($file in $files) {
  # Check if the file extension is .vert, .frag or .comp
  if ($file.Extension -in @(".vert", ".frag", ".comp")) {
    # Replace the . with an underscore and append .spv
    $newName = $file.Name.Replace(".", "_") + ".spv"
    # Rename the file
    &glslc .\shaders\$file -o ./shaders/$newName
  }
}

