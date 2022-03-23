import queue
import threading


def target(q: queue.Queue):
    while True:
        i = q.get()
        print(i)
        q.task_done()


def create_writer_thread() -> queue.Queue:
    """
    Creates a daemon thread that writes a row to a csv file.

    :return: Queue used for enqueueing rows for writing
    :rtype: queue.Queue
    """
    q = queue.Queue()
    threading.Thread(target=target, args=(q,), daemon=True).start()
    return q
