#!/bin/bash

# Script to find files with differences between original and refactored output

echo "Finding files with sentence count differences..."

# Find all _seams.txt files and their corresponding _seams2.txt files
for seams_file in $(find ~/gutenberg -name "*_seams.txt" | head -20); do
    seams2_file="${seams_file%_seams.txt}_seams2.txt"
    
    if [[ -f "$seams2_file" ]]; then
        # Count sentences in each file
        count1=$(wc -l < "$seams_file")
        count2=$(wc -l < "$seams2_file")
        
        if [[ $count1 -ne $count2 ]]; then
            echo "DIFFERENCE FOUND:"
            echo "  Original: $seams_file ($count1 sentences)"
            echo "  New:      $seams2_file ($count2 sentences)"
            echo "  Diff:     $((count2 - count1)) sentences"
            echo
            
            # Show the first few lines of difference
            echo "First few different lines:"
            diff "$seams_file" "$seams2_file" | head -20
            echo
            echo "=== DETAILED COMPARISON ==="
            
            # Find first differing sentence
            echo "Finding first difference..."
            diff -u "$seams_file" "$seams2_file" | head -50
            
            exit 0
        fi
    fi
done

echo "No differences found in first 20 files. Checking more..."

# Check more files if needed
for seams_file in $(find ~/gutenberg -name "*_seams.txt" | head -100); do
    seams2_file="${seams_file%_seams.txt}_seams2.txt"
    
    if [[ -f "$seams2_file" ]]; then
        count1=$(wc -l < "$seams_file")
        count2=$(wc -l < "$seams2_file")
        
        if [[ $count1 -ne $count2 ]]; then
            echo "DIFFERENCE FOUND:"
            echo "  Original: $seams_file ($count1 sentences)"
            echo "  New:      $seams2_file ($count2 sentences)"
            echo "  Diff:     $((count2 - count1)) sentences"
            exit 0
        fi
    fi
done

echo "No differences found in first 100 files."