#!/usr/bin/env python3

import json
import os

def create_curl_command(player_data):
    """Create a curl command for a single player's data."""
    # Extract BLS data
    bls = player_data['bls']
    
    # Create the JSON payload
    payload = {
        "eoa_address": player_data['pub'],
        "g1_x": bls['g1_x'],
        "g1_y": bls['g1_y'],
        "g2_x_a": bls['g2_x_0'],
        "g2_x_b": bls['g2_x_1'],
        "g2_y_a": bls['g2_y_0'],
        "g2_y_b": bls['g2_y_1'],
        "private_key": bls['priv_key']
    }
    
    # Create the curl command
    curl_cmd = (
        f"curl -X POST http://localhost:8080/api/keys \\\n"
        f"  -H 'Content-Type: application/json' \\\n"
        f"  -d '{json.dumps(payload)}'"
    )
    
    return curl_cmd

def main():
    # Read the players.json file
    script_dir = os.path.dirname(os.path.abspath(__file__))
    json_path = os.path.join(script_dir, 'players.json')
    
    with open(json_path, 'r') as f:
        players = json.load(f)
    
    # Create curl commands for each player
    print("# Commands to populate the database with player data")
    print("# Run these commands after starting the web server")
    print()
    
    for player_name, player_data in players.items():
        print(f"# Adding {player_name}'s key pair")
        curl_cmd = create_curl_command(player_data)
        print(curl_cmd)
        print("\nExecuting command...")
        os.system(curl_cmd)
        print()
        

if __name__ == '__main__':
    main() 