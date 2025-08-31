#!/usr/bin/env python3
"""
Fix unstable let expressions in Rust code.
Converts patterns like:
  if let Some(x) = a && let Some(y) = b { ... }
to:
  if let Some(x) = a {
      if let Some(y) = b { ... }
  }
"""

import os
import re
import sys

def fix_let_expressions_in_file(filepath):
    """Fix let expressions in a single file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern 1: if let ... && let ... 
        # This is the most common unstable pattern
        pattern1 = re.compile(
            r'if\s+let\s+([^=]+)\s*=\s*([^&]+)\s*&&\s*let\s+([^=]+)\s*=\s*([^{]+)\s*\{',
            re.MULTILINE
        )
        
        def replace_double_let(match):
            let1_pattern = match.group(1).strip()
            let1_expr = match.group(2).strip()
            let2_pattern = match.group(3).strip()
            let2_expr = match.group(4).strip()
            
            return f'if let {let1_pattern} = {let1_expr} {{\n        if let {let2_pattern} = {let2_expr} {{'
        
        content = pattern1.sub(replace_double_let, content)
        
        # Pattern 2: if let ... && condition
        pattern2 = re.compile(
            r'if\s+let\s+([^=]+)\s*=\s*([^&]+)\s*&&\s+([^{]+)\s*\{',
            re.MULTILINE
        )
        
        def replace_let_and_condition(match):
            let_pattern = match.group(1).strip()
            let_expr = match.group(2).strip()
            condition = match.group(3).strip()
            
            return f'if let {let_pattern} = {let_expr} {{\n        if {condition} {{'
        
        content = pattern2.sub(replace_let_and_condition, content)
        
        # Pattern 3: if condition && let ...
        pattern3 = re.compile(
            r'if\s+([^&]+)\s*&&\s*let\s+([^=]+)\s*=\s*([^{]+)\s*\{',
            re.MULTILINE
        )
        
        def replace_condition_and_let(match):
            condition = match.group(1).strip()
            let_pattern = match.group(2).strip()
            let_expr = match.group(3).strip()
            
            return f'if {condition} {{\n        if let {let_pattern} = {let_expr} {{'
        
        content = pattern3.sub(replace_condition_and_let, content)
        
        # Only write if content changed
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {filepath}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Fix let expressions in all Rust files."""
    src_dir = "src"
    fixed_count = 0
    
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                if fix_let_expressions_in_file(filepath):
                    fixed_count += 1
    
    print(f"Fixed {fixed_count} files")

if __name__ == "__main__":
    main()
