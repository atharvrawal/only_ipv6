import asyncio
import websockets
import json

peers = {}

async def handler(websocket):
    print("Client connected")
    try:
        async for message in websocket:
            data = json.loads(message)
            msg_type = data.get("type")

            if msg_type == "register":
                username = data["username"]
                del data["username"]
                peers[username] = data
                print(peers)
                await websocket.send("Registered")  
            elif msg_type == "get_peers":
                await websocket.send(json.dumps(peers))
            else:
                await websocket.send("unknown")
    except websockets.exceptions.ConnectionClosedOK:
        print("Client disconnected gracefully")
    except websockets.exceptions.ConnectionClosedError:
        print("Client disconnected with error")

# Entry point to start the server
async def main():
    async with websockets.serve(handler, "0.0.0.0", 8765):
        print("WebSocket server started on ws://0.0.0.0:8765")
        await asyncio.Future()  

if __name__ == "__main__":
    asyncio.run(main())
