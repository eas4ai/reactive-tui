#!/usr/bin/env python3
"""
Fix remaining unstable let expressions in Rust code.
This script specifically targets the remaining patterns that weren't caught by the first script.
"""

import os
import re
import sys

def fix_let_expressions_in_file(file_path):
    """Fix unstable let expressions in a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern 1: if let ... && let ... && let ... {
        # Convert to nested if statements
        pattern1 = re.compile(
            r'(\s*)if let ([^&]+?)\s*&&\s*let ([^&]+?)\s*&&\s*let ([^{]+?)\s*\{',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_triple_let(match):
            indent = match.group(1)
            let1 = match.group(2).strip()
            let2 = match.group(3).strip()
            let3 = match.group(4).strip()
            
            return f"""{indent}if let {let1} {{
{indent}    if let {let2} {{
{indent}        if let {let3} {{"""
        
        content = pattern1.sub(replace_triple_let, content)
        
        # Pattern 2: if let ... && let ... {
        # Convert to nested if statements
        pattern2 = re.compile(
            r'(\s*)if let ([^&]+?)\s*&&\s*let ([^{]+?)\s*\{',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_double_let(match):
            indent = match.group(1)
            let1 = match.group(2).strip()
            let2 = match.group(3).strip()
            
            return f"""{indent}if let {let1} {{
{indent}    if let {let2} {{"""
        
        content = pattern2.sub(replace_double_let, content)
        
        # Pattern 3: Complex patterns with conditions mixed in
        # if condition && let ... && condition {
        pattern3 = re.compile(
            r'(\s*)if ([^&]+?)\s*&&\s*let ([^&]+?)\s*&&\s*([^{]+?)\s*\{',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_mixed_condition_let(match):
            indent = match.group(1)
            cond1 = match.group(2).strip()
            let_expr = match.group(3).strip()
            cond2 = match.group(4).strip()
            
            return f"""{indent}if {cond1} {{
{indent}    if let {let_expr} {{
{indent}        if {cond2} {{"""
        
        content = pattern3.sub(replace_mixed_condition_let, content)
        
        # Pattern 4: if let ... && condition {
        pattern4 = re.compile(
            r'(\s*)if let ([^&]+?)\s*&&\s*([^{]+?)\s*\{',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_let_condition(match):
            indent = match.group(1)
            let_expr = match.group(2).strip()
            condition = match.group(3).strip()
            
            # Skip if this looks like it was already processed
            if 'let' in condition and '&&' in condition:
                return match.group(0)
            
            return f"""{indent}if let {let_expr} {{
{indent}    if {condition} {{"""
        
        content = pattern4.sub(replace_let_condition, content)
        
        # Pattern 5: Complex tuple destructuring in let expressions
        pattern5 = re.compile(
            r'(\s*)if let \(\s*([^)]+?)\s*\) = \([^)]+?\)\s*&&\s*([^{]+?)\s*\{',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_tuple_let(match):
            indent = match.group(1)
            tuple_pattern = match.group(2).strip()
            condition = match.group(3).strip()
            
            # For complex tuple patterns, we need to be more careful
            return f"""{indent}if let ({tuple_pattern}) = (&self.patches[i], &self.patches[i + 1]) {{
{indent}    if {condition} {{"""
        
        content = pattern5.sub(replace_tuple_let, content)
        
        # Now we need to fix the closing braces
        # Count how many extra closing braces we need to add
        if content != original_content:
            # This is a simple heuristic - for each conversion, we need extra closing braces
            # We'll add them at the end of functions/blocks
            
            # Find function endings and add extra braces
            lines = content.split('\n')
            new_lines = []
            brace_debt = 0
            
            for i, line in enumerate(lines):
                new_lines.append(line)
                
                # Count opening braces we added
                if 'if let' in line and '{' in line:
                    # Check if this was one of our replacements
                    if i > 0 and ('if let' in lines[i-1] or 'if ' in lines[i-1]):
                        brace_debt += 1
                
                # Add closing braces at function/block endings
                stripped = line.strip()
                if stripped == '}' and brace_debt > 0:
                    # Add extra closing braces
                    indent = len(line) - len(line.lstrip())
                    for _ in range(brace_debt):
                        new_lines.append(' ' * indent + '}')
                    brace_debt = 0
            
            content = '\n'.join(new_lines)
        
        # Write back if changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function to process all Rust files."""
    rust_files = []
    
    # Find all .rs files
    for root, dirs, files in os.walk('src'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    fixed_files = []
    
    for file_path in rust_files:
        if fix_let_expressions_in_file(file_path):
            fixed_files.append(file_path)
            print(f"Fixed: {file_path}")
    
    if fixed_files:
        print(f"Fixed {len(fixed_files)} files")
    else:
        print("No files needed fixing")

if __name__ == "__main__":
    main()
