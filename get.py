from bs4 import BeautifulSoup as bs
import requests
import json
import websocket


def get_token():
    content = requests.get("https://www.reddit.com/r/place/")
    soup = bs(content.content, features="html.parser")

    data = soup.find(id="data")
    data = data.text
    if data[-1] == ";":
        data = data[:-1]

    beginning = data.find("{")
    data = data[beginning:]

    data = json.loads(data)
    token = data["user"]["session"]["accessToken"]
    return token


query = """
subscription replace($input: SubscribeInput!) {
  subscribe(input: $input) {
    id
    ... on BasicMessage {
      data {
        __typename
        ... on FullFrameMessageData {
          __typename
          name
          timestamp
        }
      }
      __typename
    }
    __typename
  }
}
"""


def get_image_url(token):
    ws = websocket.create_connection("wss://gql-realtime-2.reddit.com/query")
    auth = json.dumps({
        'type': 'connection_init',
        'payload': {
            'Authorization': 'Bearer ' + token
        }
    })
    ws.send(auth)
    id = '1'
    ws.send(json.dumps({'id': id, 'type': 'start', 'payload': {'variables': {'input': {'channel': {
            'teamOwner': 'AFD2022', 'category': 'CANVAS', 'tag': '0'}}}, 'extensions': {}, 'operationName': 'replace', 'query': query}}))
    name = None
    while True:
        d = json.loads(ws.recv())
        if 'id' in d and d['id'] == id:
            data = d['payload']['data']['subscribe']['data']
            if data['__typename'] == 'FullFrameMessageData':
                name = data['name']
                break
    ws.close()
    return name


if __name__ == "__main__":
    import os
    import time
    import math

    output_dir = "frames"
    
    os.makedirs(output_dir, exist_ok=True)

    token = get_token()
    url = get_image_url(token)

    filename = str(math.floor(time.time())) + ".png"
    path = os.path.join(output_dir, filename)

    req = requests.get(url)
    with open(path, "wb") as f:
        f.write(req.content)