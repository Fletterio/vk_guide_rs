#!/bin/bash

# Iterate over all files in the current directory
for file in ./shaders/*
do
    # Check if the file ends with .vert, .frag, or .comp
    if [[ $file == *.vert ]] || [[ $file == *.frag ]] || [[ $file == *.comp ]]
    then
        # Get the base name of the file without extension
        base_name=$(basename "$file" .${file##*.})

        # Replace the . with _ in the extension
        new_ext=$(echo ${file##*.} | sed 's/\./_/')
        echo $new_ext

        # Form the new file name and append .spv
        new_file="${base_name}_${new_ext}.spv"

        glslc $file -o ./shaders/$new_file
    fi
done



