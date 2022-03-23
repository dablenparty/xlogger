from datetime import datetime


def get_file_safe_date_string() -> str:
    """
    Returns a string that is safe to use as a filename.
    """
    return datetime.now().strftime("%Y-%m-%d %H_%M_%S")
