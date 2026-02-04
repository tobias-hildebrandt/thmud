use crate::{
    Tick,
    client::client_handle_message,
    config::SyncTestConfig,
    messages::{MessageQueue, MessageToClient, MessageToServer},
    server::{WorldSyncState, send_updates, server_handle_message},
    world::{TwoWorldDisplay, WorldCells},
};

pub fn run_sync_test(config: &SyncTestConfig) {
    /*

    "server" and "client" that are driven by ticks
    - no separate threads or anything fancy
    - directly manipulate "world" stored in variables

     */

    let mut server_world = WorldCells::new_random(config.world_size);
    let mut sync_states = WorldSyncState::new_default(config.world_size);
    let mut server_queue = MessageQueue::<MessageToServer>::new();

    let mut client_world = WorldCells::new_default(config.world_size);
    let mut client_queue = MessageQueue::<MessageToClient>::new();

    let mut tick: Tick = Tick(0);

    loop {
        tracing::info!("tick {tick:?}");

        // mutate server world
        let mutations = config.mutations_per_tick.get();
        tracing::debug!("mutations this tick: {mutations}");
        for _ in 0..mutations {
            server_world.random_mutation();
        }

        // server handle messages
        while let Some(message) = server_queue.try_pop(tick) {
            server_handle_message(&mut sync_states, message);
        }

        // server send changes
        {
            let message_to_client = send_updates(
                tick,
                &server_world,
                &mut sync_states,
                config.num_sync_updates,
                &config.center.0,
            );
            let updates_str = message_to_client
                .updates
                .iter()
                .fold(String::new(), |accum, next| {
                    format!("{}, {:?}", accum, next)
                });
            tracing::debug!("server sending updates: {}", updates_str);
            client_queue.push(message_to_client, Tick(tick.0 + config.latency.get()));
        }

        // client handle messages and sends acks
        while let Some(message) = client_queue.try_pop(tick) {
            let message_to_server = client_handle_message(&mut client_world, message);
            // client sends acks
            server_queue.push(message_to_server, Tick(tick.0 + config.latency.get()));
        }

        // print states
        // tracing::info!("server:\n{}\nclient:\n{}", server_world, client_world);
        tracing::info!(
            "\n{}",
            TwoWorldDisplay {
                server: &server_world,
                client: &client_world
            }
        );
        tracing::debug!("messages in flight to server: {:?}", server_queue);
        tracing::debug!("messages in flight to client: {:?}", client_queue);

        // compare states, accumulate stats

        // wait for tick
        config.wait_for_tick.wait();
        tick.0 += 1;
    }
}
