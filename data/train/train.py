MARGIN=50
BATCHSIZE=32
CLASSES = 32

SIZE=MARGIN*2+1

import tensorflow as tf
from PIL import Image

from tensorflow.keras import layers
import tensorflow.keras as keras

def _decode(record_bytes):
    o = tf.io.parse_single_example(record_bytes, {
        'window': tf.io.FixedLenFeature([SIZE, SIZE, 3], tf.float32),
        'index': tf.io.FixedLenFeature([1], tf.int64),
    })
    return o['window'], o['index']

dataset = tf.data.TFRecordDataset('dataset.tfrecord').map(_decode).batch(BATCHSIZE)
validation = tf.data.TFRecordDataset('validation.tfrecord').map(_decode).batch(BATCHSIZE)


def get_model():
    inputs = keras.Input(shape=(SIZE, SIZE, 3))
    #101
    x = layers.Conv2D(filters=64, kernel_size=(3,3), activation="relu")(inputs) #99
    x = layers.MaxPooling2D(pool_size=(3, 3))(x)
    x = layers.Conv2D(filters=64, kernel_size=(3,3), activation="relu")(x)
    x = layers.MaxPooling2D(pool_size=(3, 3))(x)
    #x = layers.Conv2D(filters=64, kernel_size=(3,3), activation="relu")(x)
    x = layers.GlobalAveragePooling2D()(x)
    x = layers.Dense((64+32)//2, activation="relu")(x)
    outputs = layers.Dense(CLASSES, activation="softmax")(x)
    model = keras.Model(inputs=inputs, outputs=outputs)
    return model
    
model = get_model()


model.compile(optimizer='adam',
              loss=keras.losses.SparseCategoricalCrossentropy(),
              metrics=['accuracy'])
model.summary()


checkpoint_callback = keras.callbacks.ModelCheckpoint(
    filepath='chkpnt',
)

tensorboard_callback = tf.keras.callbacks.TensorBoard(log_dir="logs")

model.fit(dataset, epochs=10, callbacks=[checkpoint_callback, tensorboard_callback], validation_data=validation)