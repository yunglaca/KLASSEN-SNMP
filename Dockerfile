FROM alpine:latest

# тестовый докер контейнер для v3 
RUN apk add --no-cache net-snmp net-snmp-tools

# username: myuser
# auth:    SHA, password "myauthpass"
# privacy: AES, password "myprivpass"
RUN mkdir -p /var/lib/net-snmp && \
    echo 'createUser myuser SHA "myauthpass" AES "myprivpass"' >> /var/lib/net-snmp/snmpd.conf && \
    echo 'rwuser myuser authpriv' > /etc/snmp/snmpd.conf && \
    echo 'syslocation "Test"' >> /etc/snmp/snmpd.conf && \
    echo 'syscontact "test"' >> /etc/snmp/snmpd.conf

EXPOSE 161/udp

CMD ["snmpd", "-f", "-Lo"]
