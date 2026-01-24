#!/usr/bin/env python3
"""
lamport_clock.py
Implementation of Lamport logical clocks for distributed systems.

Usage:
    python lamport_clock.py --demo
    python lamport_clock.py --simulate 3 10
"""

import argparse
import random
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple
from enum import Enum
import json


class EventType(Enum):
    LOCAL = "local"
    SEND = "send"
    RECEIVE = "receive"


@dataclass
class Event:
    """An event in a distributed system."""
    process_id: str
    event_type: EventType
    timestamp: int
    description: str
    message_id: Optional[str] = None
    
    def __str__(self):
        return f"[{self.process_id}@{self.timestamp}] {self.event_type.value}: {self.description}"


@dataclass
class Message:
    """A message sent between processes."""
    id: str
    sender: str
    receiver: str
    timestamp: int
    content: str


class LamportClock:
    """
    Lamport logical clock implementation.
    
    Rules:
    1. Before any event, increment clock
    2. When sending, include clock value in message
    3. When receiving, set clock to max(local, received) + 1
    """
    
    def __init__(self, process_id: str):
        self.process_id = process_id
        self.time = 0
        self.events: List[Event] = []
    
    def local_event(self, description: str) -> Event:
        """Record a local event."""
        self.time += 1
        event = Event(
            process_id=self.process_id,
            event_type=EventType.LOCAL,
            timestamp=self.time,
            description=description
        )
        self.events.append(event)
        return event
    
    def send(self, receiver: str, content: str) -> Tuple[Event, Message]:
        """Send a message to another process."""
        self.time += 1
        
        message = Message(
            id=f"{self.process_id}-{self.time}",
            sender=self.process_id,
            receiver=receiver,
            timestamp=self.time,
            content=content
        )
        
        event = Event(
            process_id=self.process_id,
            event_type=EventType.SEND,
            timestamp=self.time,
            description=f"Send to {receiver}: {content}",
            message_id=message.id
        )
        self.events.append(event)
        
        return event, message
    
    def receive(self, message: Message) -> Event:
        """Receive a message from another process."""
        # Lamport rule: max(local, received) + 1
        self.time = max(self.time, message.timestamp) + 1
        
        event = Event(
            process_id=self.process_id,
            event_type=EventType.RECEIVE,
            timestamp=self.time,
            description=f"Receive from {message.sender}: {message.content}",
            message_id=message.id
        )
        self.events.append(event)
        
        return event


class DistributedSystem:
    """Simulates a distributed system with Lamport clocks."""
    
    def __init__(self, process_ids: List[str]):
        self.processes: Dict[str, LamportClock] = {
            pid: LamportClock(pid) for pid in process_ids
        }
        self.pending_messages: List[Message] = []
        self.delivered_messages: List[Message] = []
    
    def local_event(self, process_id: str, description: str) -> Event:
        """Execute a local event on a process."""
        return self.processes[process_id].local_event(description)
    
    def send_message(self, sender: str, receiver: str, content: str) -> Message:
        """Send a message (goes to pending queue)."""
        event, message = self.processes[sender].send(receiver, content)
        self.pending_messages.append(message)
        return message
    
    def deliver_message(self, message: Message) -> Event:
        """Deliver a pending message to its receiver."""
        if message in self.pending_messages:
            self.pending_messages.remove(message)
            self.delivered_messages.append(message)
            return self.processes[message.receiver].receive(message)
        raise ValueError(f"Message {message.id} not in pending queue")
    
    def deliver_all_pending(self):
        """Deliver all pending messages (in random order to simulate network)."""
        while self.pending_messages:
            # Random delivery order simulates network non-determinism
            message = random.choice(self.pending_messages)
            self.deliver_message(message)
    
    def get_global_history(self) -> List[Event]:
        """Get all events sorted by Lamport timestamp."""
        all_events = []
        for process in self.processes.values():
            all_events.extend(process.events)
        
        # Sort by timestamp, then by process_id for ties
        return sorted(all_events, key=lambda e: (e.timestamp, e.process_id))
    
    def print_timeline(self):
        """Print timeline visualization."""
        process_ids = sorted(self.processes.keys())
        
        # Header
        print("\n" + "=" * 60)
        print("LAMPORT CLOCK TIMELINE")
        print("=" * 60)
        
        # Column headers
        header = "Time |"
        for pid in process_ids:
            header += f" {pid:^12} |"
        print(header)
        print("-" * len(header))
        
        # Group events by timestamp
        history = self.get_global_history()
        max_time = max(e.timestamp for e in history) if history else 0
        
        for t in range(1, max_time + 1):
            row = f" {t:2}  |"
            for pid in process_ids:
                events_at_t = [e for e in history 
                              if e.timestamp == t and e.process_id == pid]
                if events_at_t:
                    e = events_at_t[0]
                    symbol = {"local": "●", "send": "→", "receive": "←"}[e.event_type.value]
                    row += f" {symbol:^12} |"
                else:
                    row += f" {'':^12} |"
            print(row)
        
        print("=" * 60)
        print("\nLegend: ● = local event, → = send, ← = receive")
    
    def verify_causality(self) -> List[str]:
        """Verify that Lamport timestamps respect causality."""
        violations = []
        
        for message in self.delivered_messages:
            send_event = None
            recv_event = None
            
            for e in self.processes[message.sender].events:
                if e.message_id == message.id and e.event_type == EventType.SEND:
                    send_event = e
                    break
            
            for e in self.processes[message.receiver].events:
                if e.message_id == message.id and e.event_type == EventType.RECEIVE:
                    recv_event = e
                    break
            
            if send_event and recv_event:
                if recv_event.timestamp <= send_event.timestamp:
                    violations.append(
                        f"Causality violation: send@{send_event.timestamp} -> "
                        f"receive@{recv_event.timestamp}"
                    )
        
        return violations


def demo():
    """Demonstrate Lamport clocks with a classic example."""
    print("=" * 60)
    print("LAMPORT CLOCK DEMONSTRATION")
    print("=" * 60)
    print("\nScenario: Three processes exchanging messages\n")
    
    system = DistributedSystem(["A", "B", "C"])
    
    # Process A does some work
    system.local_event("A", "Start computation")
    
    # A sends to B
    msg1 = system.send_message("A", "B", "Hello B")
    
    # B does local work before receiving
    system.local_event("B", "Initialize")
    system.local_event("B", "Process data")
    
    # B receives A's message
    system.deliver_message(msg1)
    
    # B sends to C
    msg2 = system.send_message("B", "C", "Forward to C")
    
    # C does local work
    system.local_event("C", "Startup")
    
    # C receives B's message
    system.deliver_message(msg2)
    
    # C sends back to A
    msg3 = system.send_message("C", "A", "Response to A")
    
    # A does more work
    system.local_event("A", "Continue processing")
    
    # A receives C's response
    system.deliver_message(msg3)
    
    # Print results
    system.print_timeline()
    
    print("\nEvent History (Lamport Order):")
    print("-" * 40)
    for event in system.get_global_history():
        print(event)
    
    # Verify causality
    violations = system.verify_causality()
    if violations:
        print("\n⚠️  Causality violations detected:")
        for v in violations:
            print(f"  - {v}")
    else:
        print("\n✅ All causality constraints satisfied!")


def simulate(num_processes: int, num_events: int):
    """Run a random simulation."""
    process_ids = [f"P{i}" for i in range(num_processes)]
    system = DistributedSystem(process_ids)
    
    print(f"\nSimulating {num_processes} processes with ~{num_events} events...")
    
    for _ in range(num_events):
        # Random action
        action = random.choice(["local", "send"])
        sender = random.choice(process_ids)
        
        if action == "local":
            system.local_event(sender, f"Work")
        else:
            receiver = random.choice([p for p in process_ids if p != sender])
            system.send_message(sender, receiver, "msg")
    
    # Deliver all messages
    system.deliver_all_pending()
    
    system.print_timeline()
    
    violations = system.verify_causality()
    print(f"\nCausality violations: {len(violations)}")


def main():
    parser = argparse.ArgumentParser(description="Lamport Clock Demonstration")
    parser.add_argument("--demo", action="store_true", help="Run demonstration")
    parser.add_argument("--simulate", nargs=2, type=int, 
                       metavar=("PROCESSES", "EVENTS"),
                       help="Simulate N processes with M events")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    
    args = parser.parse_args()
    
    if args.simulate:
        simulate(args.simulate[0], args.simulate[1])
    else:
        demo()


if __name__ == "__main__":
    main()
