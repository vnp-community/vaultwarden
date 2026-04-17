import re
import os

files = [
    "specs/crs/v1/solutions/tasks/TASKS-SOL-005.md",
    "specs/crs/v1/solutions/tasks/TASKS-SOL-006.md",
    "specs/crs/v1/solutions/tasks/TASKS-SOL-007.md",
    "specs/crs/v1/solutions/tasks/TASKS-SOL-008.md",
    "specs/crs/v1/solutions/tasks/TASKS-SOL-009.md",
    "specs/crs/v1/solutions/tasks/TASKS-SOL-010.md",
    "specs/crs/v1/solutions/tasks/TASKS-SOL-011.md"
]

def check_task(desc):
    # A simple heuristic: check if the mentioned files exist.
    # If the file is mentioned as "mới" (new), check if it exists on disk.
    # If it's an existing file being "Modified", we check if the function or model exists.
    files_match = re.search(r'- \*\*File\*\*: (.*)', desc)
    if files_match:
        paths = files_match.group(1).replace('`', '').split(', ')
        for p in paths:
            clean_path = p.split(' ')[0] # remove (mới) etc
            if clean_path.startswith('src/'):
                if not os.path.exists(clean_path):
                    return False
                
                # Check for "Implement" lines
                impl_match = re.search(r'- \*\*Tên\*\*: (.*)', desc)
                if impl_match:
                    name = impl_match.group(1).lower()
                    if 'api_key_v2.rs' in clean_path and not os.path.exists('src/db/models/api_key_v2.rs'):
                       return False
                
                # Further heuristics can be added, but just checking if the target file exists
                # is usually enough to see if it's scaffolded. But a scaffold is not complete.
                # Let's see if the file has "TODO:" or similar scaffold marks
                if os.path.exists(clean_path):
                    with open(clean_path, 'r') as f:
                        content = f.read()
                        if 'TODO: Implement' in content: 
                            return False
    
    return False

for file in files:
    with open(file, 'r') as f:
        content = f.read()
    
    # We aren't doing complex analysis, so let's just leave the ones we haven't done as [ ]
    # Actually, as the assistant, I know I only recently did SOL-005 Phase 1&2, SOL-009 configs, SOL-011 core, SOL-003.
    # I can just leave this script to print the statuses.
    pass

