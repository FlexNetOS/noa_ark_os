use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use ark_os_core::*;
use tokio::runtime::Runtime;
use uuid::Uuid;
use std::time::Duration;

fn bench_message_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let broker = rt.block_on(async {
        let broker = MessageBroker::new().await.unwrap();
        broker.start().await.unwrap();
        broker
    });
    
    let mut group = c.benchmark_group("message_throughput");
    
    for message_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*message_count as u64));
        group.bench_with_input(
            BenchmarkId::new("messages", message_count),
            message_count,
            |b, &message_count| {
                b.to_async(&rt).iter(|| async {
                    let agent1_id = Uuid::new_v4();
                    let agent2_id = Uuid::new_v4();
                    
                    let _channel1 = broker.register_agent(agent1_id).await.unwrap();
                    let _channel2 = broker.register_agent(agent2_id).await.unwrap();
                    
                    for _ in 0..message_count {
                        let message = Message::new(
                            agent1_id,
                            Some(agent2_id),
                            MessageType::Request,
                            communication::Priority::Normal,
                            serde_json::json!({"benchmark": true}),
                        );
                        
                        broker.send_message(message).await.unwrap();
                    }
                    
                    black_box(message_count)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_broadcast_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let broker = rt.block_on(async {
        let broker = MessageBroker::new().await.unwrap();
        broker.start().await.unwrap();
        broker
    });
    
    let mut group = c.benchmark_group("broadcast_performance");
    
    for agent_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("agents", agent_count),
            agent_count,
            |b, &agent_count| {
                b.to_async(&rt).iter_batched(
                    || {
                        // Setup phase: register agents
                        rt.block_on(async {
                            let mut channels = Vec::new();
                            for _ in 0..agent_count {
                                let agent_id = Uuid::new_v4();
                                let channel = broker.register_agent(agent_id).await.unwrap();
                                channels.push((agent_id, channel));
                            }
                            channels
                        })
                    },
                    |agents| async move {
                        // Benchmark phase: broadcast message
                        let broadcaster_id = agents[0].0;
                        let broadcast_message = Message::new(
                            broadcaster_id,
                            None,
                            MessageType::Broadcast,
                            communication::Priority::Normal,
                            serde_json::json!({"broadcast_benchmark": true}),
                        );
                        
                        broker.broadcast(broadcast_message).await.unwrap();
                        black_box(agent_count)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_concurrent_messaging(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let broker = rt.block_on(async {
        let broker = MessageBroker::new().await.unwrap();
        broker.start().await.unwrap();
        broker
    });
    
    c.bench_function("concurrent_messaging", |b| {
        b.to_async(&rt).iter(|| async {
            let agent_count = 100;
            let messages_per_agent = 10;
            
            // Register agents
            let mut agents = Vec::new();
            for _ in 0..agent_count {
                let agent_id = Uuid::new_v4();
                let _channel = broker.register_agent(agent_id).await.unwrap();
                agents.push(agent_id);
            }
            
            // Send messages concurrently
            let mut handles = Vec::new();
            
            for &sender_id in &agents {
                for &receiver_id in &agents {
                    if sender_id != receiver_id {
                        for msg_idx in 0..messages_per_agent {
                            let broker_clone = &broker;
                            let handle = tokio::spawn(async move {
                                let message = Message::new(
                                    sender_id,
                                    Some(receiver_id),
                                    MessageType::Request,
                                    communication::Priority::Normal,
                                    serde_json::json!({"msg_idx": msg_idx}),
                                );
                                
                                broker_clone.send_message(message).await.unwrap();
                            });
                            handles.push(handle);
                        }
                    }
                }
            }
            
            // Wait for all messages to be sent
            for handle in handles {
                handle.await.unwrap();
            }
            
            black_box(agent_count * (agent_count - 1) * messages_per_agent)
        })
    });
}

fn bench_message_queue_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let broker = rt.block_on(async {
        let broker = MessageBroker::new().await.unwrap();
        broker.start().await.unwrap();
        broker
    });
    
    c.bench_function("queue_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let agent_id = Uuid::new_v4();
            let mut channel = broker.register_agent(agent_id).await.unwrap();
            
            // Send messages to fill queue
            for i in 0..100 {
                let message = Message::new(
                    Uuid::new_v4(),
                    Some(agent_id),
                    MessageType::Request,
                    communication::Priority::Normal,
                    serde_json::json!({"queue_test": i}),
                );
                
                broker.send_message(message).await.unwrap();
            }
            
            // Receive messages to empty queue
            let mut received_count = 0;
            while let Some(_message) = tokio::time::timeout(
                Duration::from_millis(1),
                channel.receive_message()
            ).await.ok().flatten() {
                received_count += 1;
                if received_count >= 100 {
                    break;
                }
            }
            
            black_box(received_count)
        })
    });
}

fn bench_message_priority_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let broker = rt.block_on(async {
        let broker = MessageBroker::new().await.unwrap();
        broker.start().await.unwrap();
        broker
    });
    
    c.bench_function("priority_handling", |b| {
        b.to_async(&rt).iter(|| async {
            let sender_id = Uuid::new_v4();
            let receiver_id = Uuid::new_v4();
            let _channel = broker.register_agent(receiver_id).await.unwrap();
            
            // Send messages with different priorities
            let priorities = vec![
                communication::Priority::Emergency,
                communication::Priority::Critical,
                communication::Priority::High,
                communication::Priority::Normal,
                communication::Priority::Low,
            ];
            
            for (i, priority) in priorities.iter().enumerate() {
                let message = Message::new(
                    sender_id,
                    Some(receiver_id),
                    MessageType::Request,
                    priority.clone(),
                    serde_json::json!({"priority_test": i}),
                );
                
                broker.send_message(message).await.unwrap();
            }
            
            black_box(priorities.len())
        })
    });
}

criterion_group!(
    benches,
    bench_message_throughput,
    bench_broadcast_performance,
    bench_concurrent_messaging,
    bench_message_queue_operations,
    bench_message_priority_handling
);
criterion_main!(benches);