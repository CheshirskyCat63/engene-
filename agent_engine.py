# filename: agent_engine.py
import os
from pyautogen.agent import AutoGenAgent  # важно: из pyautogen.agent

# Настройка рабочего каталога вашего движка
WORK_DIR = r"C:\Users\CheCat\project\engine"

# Инструменты агента
def read_file(filename: str):
    path = os.path.join(WORK_DIR, filename)
    if os.path.exists(path):
        with open(path, "r", encoding="utf-8") as f:
            return f.read()
    return f"File not found: {filename}"

def write_file(filename: str, content: str):
    path = os.path.join(WORK_DIR, filename)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    return f"Written to {filename}"

def run_command(cmd: str):
    return os.popen(cmd).read()

TOOLS = [
    {"name": "ReadFile", "func": read_file, "description": "Read any file from the engine project"},
    {"name": "WriteFile", "func": write_file, "description": "Write content to a file in the engine project"},
    {"name": "RunCommand", "func": run_command, "description": "Run any terminal command in Windows"},
]

# Инициализация агента
agent = AutoGenAgent(
    name="EngineAgent",
    tools=TOOLS,
    temperature=0,
    verbose=True
)

# Задача агента
TASK = """
Ты автономный агент. Твоя цель:
1. Проверить весь проект движка на устаревшие API (deprecated).
2. Запустить тесты (pytest).
3. Профилировать main.py.
4. Исправлять устаревшие API, если возможно.
5. Создать отчёт о проблемах и исправлениях.
Работай как курсор, делай всё сам, не спрашивая.
"""

# Запуск агента
agent.run(TASK)