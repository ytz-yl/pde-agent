#!/usr/bin/env bash
# Start/stop/status Neo4j for the PDE knowledge base dev environment.
# Usage: ./start-neo4j.sh [start|stop|status|restart]
set -e

export JAVA_HOME=/opt/java/jdk-21.0.7+6
mkdir -p /tmp/neo4j-run

NEO4J=/usr/share/neo4j/bin/neo4j
CMD=${1:-start}

case "$CMD" in
  start|stop|status|restart|console)
    exec "$NEO4J" "$CMD"
    ;;
  *)
    echo "Usage: $0 [start|stop|status|restart|console]"
    exit 1
    ;;
esac
