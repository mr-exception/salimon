# Legacy Salimon hub notes

> Archived from the original Salimon hub. This entity/buffer design predates the current space game and is not a game specification. The original page contained two diagrams (neuron and cycle) that were not included in this Markdown export; their behavior is described below.

## Environment
an environment is a workspace where all entities are serving together. it has two main parts:
### Buffer
is an array of variables in memory of the host machine. each slot can have different roles from `readWrite`, `onlyRead` and `onlyWrite` , buffers are defined by an identical address or `bufferId` and can’t hold a value in 1 byte. this value for entities is considered a 8bit unsigned integer (0-255) and has all native behaviors of a `uint8`  such as overflow and size.
each buffer slot is unique based on it’s usage and the knowledge about buffer slots is shared between all entities.
### Logics
is a collection of entities containing one or more `logic unit` . each logic unit performs a single operation on buffer. it’s similar to the neuron in neural network context.

logic units can have at least 0 inputs mapped to a slots in buffer array. each input multiplies the mapped slot by `w` or weight and adds it to neuron value. neuron value is a 1 byte unsigned integer and will overflow if its value exceeds 255.
$$
w_{t} = w_{0} b_{0} + w_{1}b_{1} + ... + w_{n}b_{n}
$$
$`w_{t}`$ will be stored in a 1byte variable as neuron total weight value. then it maps into each output buffer slot with the bias defined for it.
$$
buffer_{i} = w_{t} + b_{i}
$$
$`buffer_{i}`$ as mentioned before is a 1byte variable and any exceeding results will overflow. each logic unit can have at least 1 output. they all will use the same $`w_{t}`$ for calculation
some logic unit may not have any inputs. they simply add an extra bias to mapped outputs.
## Entity
entity is a modular part of salimon network. an entity can serve on any kind of platform with some requirements. each entity is a collection of logic units to perform a task on environment. an environment can have multiple entities working in it. entities can have various kind of tasks. there are no special categories to classify these tasks but we can divide them into 3 basic goals:
1. detection: extracts a pattern and summarize it. this type of tasks includes detecting objects from and image, special patterns from reports, features and aspects from texts and other type of detection from environment buffers
2. execution: The system provides actionable data based on the buffer state. The entity itself does not directly perform any actions, but the environment has the capability to initiate jobs based on specific buffer values. This dynamic interplay between the system, entity, and environment ensures efficient resource allocation and task execution.
3. refining: In certain scenarios, more intricate tasks demand complex inputs. These tasks necessitate additional processing of the buffer data. For instance, consider a detection task that extracts individual digits from an image. However, assembling these extracted digits into a complete number or discerning whether they represent separate numbers altogether constitutes a refining process.
## Cycle
In our project, specific segments of the buffer array serve as input areas. At the start of each cycle, the environment populates these designated segments with carefully designed inputs from various sources, including sensors and signals. Regardless of the initial values stored in these buffer slots, the environment consistently overwrites them at the end of each cycle, ensuring a fresh input for subsequent processing.

Some segments of the buffer array are considered input areas. During each cycle, the environment populates these segments with carefully designed inputs from various sources, such as sensors and signals. Regardless of the initial values stored in these buffer slots, the environment overwrites them at the start of each new cycle.
## Training
Training refers to the iterative process by which an environment adjusts and enhances the capabilities of entities to achieve improved performance or quality in specific tasks. During training, these entities learn from experience, adapt to changing conditions, and refine their abilities through exposure to relevant data or stimuli.
