class DataProcessor:
    """
    A class that simulates data processing.
    """
    def __init__(self, data):
        self.data = data

    def process_data(self):
        # complex processing logic
        return [d * 2 for d in self.data]

    def _internal_method(self):
        print("This is a private method starting with underscore")
