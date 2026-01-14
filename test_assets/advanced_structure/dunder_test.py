class __SecretInternal:
    """
    Class with double underscore prefix.
    """
    def __init__(self):
        self.__hidden = True
        print(f"Secret initialized: {self.__hidden}")

    def __dunder_method__(self):
        return "impl"

def regular_func():
    """
    A regular function properly documented to satisfy linter.
    """
    pass
