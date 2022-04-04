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


frame_query = """
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

config_query = """
subscription configuration($input: SubscribeInput!) {
  subscribe(input: $input) {
    id
    ... on BasicMessage {
      data {
        __typename
        ... on ConfigurationMessageData {
          canvasConfigurations {
            index
            dx
            dy
            __typename
          }
          canvasWidth
          canvasHeight
          __typename
        }
      }
      __typename
    }
    __typename
  }
}
"""

id = 1

def send_message(ws, message):
  global id
  _id = str(id)
  id += 1
  message['id'] = _id
  ws.send(json.dumps(message))
  while True:
    d = json.loads(ws.recv())
    if 'id' in d and d['id'] == _id:
        data = d['payload']['data']['subscribe']['data']
        yield data

def get_url(ws, tag):
  messages = send_message(ws, {'type': 'start', 'payload': {'variables': {'input': {'channel': {
          'teamOwner': 'AFD2022', 'category': 'CANVAS', 'tag': str(tag)}}}, 'extensions': {}, 'operationName': 'replace', 'query': frame_query}})
  name = None
  for message in messages:
    if message['__typename'] == 'FullFrameMessageData':
      name = message['name']
      return name



def get_canvas_configs(ws):
  messages = send_message(ws, {'id': '1', 'type': 'start', 'payload': {'variables': {'input': {'channel': {'teamOwner': 'AFD2022', 'category': 'CONFIG'}}}, 'extensions': {}, 'operationName': 'configuration', 'query': config_query}})
  for message in messages:
    yield (message['canvasWidth'], message['canvasHeight'])
    for config in message['canvasConfigurations']:
      yield (config['index'], config['dx'], config['dy'], )
    return

def join_box(a, b):
  mina, maxa = a
  minb, maxb = b
  _min = tuple(min(a, b) for (a, b) in zip(mina, minb))
  _max = tuple(max(a, b) for (a, b) in zip(maxa, maxb))
  return _min, _max

def get_image_url(token):
    ws = websocket.create_connection("wss://gql-realtime-2.reddit.com/query")
    auth = json.dumps({
        'type': 'connection_init',
        'payload': {
            'Authorization': 'Bearer ' + token
        }
    })
    ws.send(auth)

    images = []

    configs = get_canvas_configs(ws)

    w, h = next(configs)

    bounding_box = None

    for (i, x, y) in configs:
      images.append((x, y, get_url(ws, i)))
      _box = ((x, y), (x+w, y+h))
      if bounding_box is None:
        bounding_box = _box
      else:
        bounding_box = join_box(bounding_box, _box)
    



    ws.close()
    return images, bounding_box


if __name__ == "__main__":
    import os
    import time
    import math
    import sys

    if "-h" in sys.argv:
      print("--join: Joins together the different parts of the image (Requires Pillow)")
      sys.exit()

    should_join = "--join" in sys.argv

    output_dir = "frames"
    
    os.makedirs(output_dir, exist_ok=True)

    token = get_token()
    images, bounding_box = get_image_url(token)



    if should_join:
      from PIL import Image
      from io import BytesIO
      #Assuming min is (0, 0)
      output_image = Image.new(size=bounding_box[1], mode="RGB")

    for i, (x, y, url) in enumerate(images):
      req = requests.get(url)
      if not should_join:
        filename = "{}_{}.png".format(math.floor(time.time()), i)
        path = os.path.join(output_dir, filename)
        with open(path, "wb") as f:
            f.write(req.content)
      else:
        part = Image.open(BytesIO(req.content))
        output_image.paste(part, (x, y))
    if should_join:
      filename = "{}.png".format(math.floor(time.time()))
      path = os.path.join(output_dir, filename)
      output_image.save(path)