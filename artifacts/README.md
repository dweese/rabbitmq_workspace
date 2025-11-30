# Messaging Topology Design

This directory contains the artifacts that define the messaging architecture for the project.

## `message_types.json`

This file is the **single source of truth** for the messaging topology. It is read by the `topology-creator` binary to declare exchanges and queues on the RabbitMQ server.

### Structure

-   **`message_types`**: An array of message categories.
    -   **`category`**: The name of the category, which is used as the exchange name (e.g., "commands", "events").
    -   **`types`**: An array of message types within that category.
        -   **`name`**: The name of the message type, which is used as the queue name and the routing key.
        -   **`durable`**: A boolean (`true` or `false`) that determines if the queue should survive a server restart.
        -   **`priority`**: A `u8` integer that specifies the maximum priority for the queue.
            -   If `priority` is present and greater than 0, the queue will be created with the `x-max-priority` argument.
            -   A value of `0` or the absence of the field means the queue will not be a priority queue.
            -   The recommended range for RabbitMQ is typically between 1 and 10.

### Categories

-   **`commands`**: These are requests to perform a specific action. They are sent to a central exchange and routed to a specific queue based on their routing key.
-   **`events`**: These are notifications that something has happened in the system. They are published to a topic exchange and can be subscribed to by multiple services.
-   **`queries`**: These are requests for information. They often include a `reply-to` property for the response.
