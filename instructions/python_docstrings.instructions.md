---
applyTo: '**/*.py'
description: 'Standard docstring format for Python files.'
---

# Docstring Style Guide

When writing or modifying code, please follow this docstring format:

## Function and Method Docstrings

### Base

All functions and methods must have docstrings with this structure:
```python
"""One-line summary of the function.

<if complex function>
More detailed description of what the function does and how it works.
</if>

Args:
    param_name (param_type): Description of parameter.
    param_name2 (param_type, optional): Description of optional parameter.
    
Returns:
    return_type: Description of return value.
    
Raises:
    ExceptionType: Description of when this exception is raised.
"""
```

If the function or method is quite long or complex, make sure to include the detailed description after the one-line summary.
This helps in understanding the purpose and functionality of the code.

### About Parameters

- Always indicate optional parameters with the format:

```python
"""
param_name (param_type, optional): Description of optional parameter. Defaults to <default_value>.
"""
```

- If the function has no parameters, you can skip the `Args` section entirely.

### About Returns

- For methods with complex return types, be explicit about the structure:

```python
"""
Returns:
    Tuple[bool, str]: A tuple containing:
        - bool: Whether the service is available
        - str: A descriptive message
"""
```

- If the function does not return anything, you can skip the `Returns` section entirely.

## Class Docstrings

All classes must have docstrings with this structure:

```python
class MyClass:
    """One-line summary of the class.

    More detailed description of the class.
    """
```

## Module Docstrings

All modules must have docstrings with this structure:

```python
"""One-line summary of the module.

More detailed description of the module.
"""
```



