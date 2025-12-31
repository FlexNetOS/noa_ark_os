#!/bin/bash

# --- Noa Ark OS Unified Notebook Setup and Run Script ---
# This script sets up a clean Python virtual environment, installs all necessary
# dependencies, and launches the Jupyter Notebook for the Noa Ark OS Unified Interface.

NOTEBOOK_PATH="noa_ark_os/notebooks/noa_ark_os_unified_interface.ipynb"
ENV_NAME="noa_ark_os_env"

echo "--- 1. Creating Python Virtual Environment ---"
python3 -m venv $ENV_NAME

if [ $? -ne 0 ]; then
    echo "ERROR: Failed to create virtual environment. Ensure python3-venv is installed."
    exit 1
fi

echo "--- 2. Activating Virtual Environment ---"
source $ENV_NAME/bin/activate

echo "--- 3. Installing Dependencies from requirements.txt ---"
pip install --upgrade pip
pip install -r requirements.txt

if [ $? -ne 0 ]; then
    echo "ERROR: Failed to install dependencies. Check requirements.txt and your network connection."
    deactivate
    exit 1
fi

echo "--- 4. Launching Jupyter Notebook ---"
echo "Please open your web browser to the URL provided below."
echo "Press Ctrl+C in this terminal to stop the Jupyter server."

# Launch the notebook, opening it directly in the browser
jupyter notebook $NOTEBOOK_PATH

# Deactivate the environment after the user stops the server
deactivate

echo "--- Setup and Run Complete ---"
echo "To clean up, you can delete the '$ENV_NAME' directory."
