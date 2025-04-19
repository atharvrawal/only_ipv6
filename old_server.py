import asyncio
import json
import websockets

peers = {}
relay_sessions = {}  # maps websocket -> peer websocket

def get_username_by_ws(ws):
    for username, info in peers.items():
        if info["websocket"] == ws:
            return username
    return None

async def signalling_handler(websocket):
    try:
        async for message in websocket:
            print("\n📡 Active users:", list(peers.keys()))

            if isinstance(message, bytes):
                # Relay binary message if in a relay session
                if websocket in relay_sessions:
                    peer_ws = relay_sessions[websocket]
                    await peer_ws.send(message)
                else:
                    print("⚠ Received binary message but not in relay")
                continue

            data = json.loads(message)
            msg_type = data.get("type")

            if msg_type == "register":
                peers[data["username"]] = {
                    "pip": data.get("pip", ""),
                    "ip": data.get("ip", ""),
                    "port": data.get("port", ""),
                    "websocket": websocket
                }
#                await websocket.send(json.dumps({"status": "registered"}))
                print(f"✅ Registered: {data['username']}")

            elif msg_type == "request_peer":
                usernames = list(peers.keys())
                await websocket.send(json.dumps(usernames))

            elif msg_type == "initiate_relay":
                target = data.get("target")
                sender = get_username_by_ws(websocket)

                if not target or target not in peers:
                    await websocket.send(json.dumps({"error": "Target user not found"}))
                else:
                    target_ws = peers[target]["websocket"]

                    # Register the session (both directions)
                    relay_sessions[websocket] = target_ws
                    relay_sessions[target_ws] = websocket

                    response = {
                        "status": "relay_initiated",
                        "type": "relay_initiated",
                        "target": target,
                        "initiator": sender,
                    }

                    await websocket.send(json.dumps(response))      # Tell sender
                    await target_ws.send(json.dumps(response))      # Tell receiver

                    print(f"🔄 Relay started between {sender} and {target}")

            elif msg_type == "relay_control" and data.get("action") == "end":
                peer_ws = relay_sessions.pop(websocket, None)
                if peer_ws:
                    await peer_ws.send(json.dumps({
                        "type": "relay_control",
                        "action": "end"
                    }))
                    relay_sessions.pop(peer_ws, None)
                    print("❌ Relay session ended.")

            elif msg_type == "peer_information":
                target = data.get("target")
                if target in peers:
                    info = peers[target]
                    await websocket.send(json.dumps({
                        "type": "peer_info",
                        "pip": info["pip"],
                        "ip": info["ip"],
                        "port": info["port"]
                    }))
                else:
                    await websocket.send(json.dumps({"error": "User not found"}))

            else:
                # Relay all other JSON messages if in a session
                if websocket in relay_sessions:
                    peer_ws = relay_sessions[websocket]
                    await peer_ws.send(message)
                else:
                    await websocket.send(json.dumps({"error": "Unknown or invalid request type"}))


    except websockets.exceptions.ConnectionClosed:
        print(f"🔌 Client disconnected.")
        # Clean up on disconnect
        username = get_username_by_ws(websocket)
        if username:
            print(f"Removing user: {username}")
            del peers[username]

        peer_ws = relay_sessions.pop(websocket, None)
        if peer_ws:
            relay_sessions.pop(peer_ws, None)
            try:
                await peer_ws.send(json.dumps({
                    "type": "relay_control",
                    "action": "end"
                }))
            except:
                pass

# Server config
server_ip = "0.0.0.0"
server_port = 8765

async def main():
    server = await websockets.serve(
        signalling_handler,
        server_ip,
        server_port,
        max_size=None  # So you can send large files
    )
    print(f"🚀 Signaling server running at {server_ip}:{server_port}")
    await asyncio.Future()  # Run forever

if __name__ == "__main__":
    asyncio.run(main())