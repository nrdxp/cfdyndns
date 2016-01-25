FROM ubuntu:latest

RUN bash -c "\
	apt-get update && \
	apt-get install libssl-dev openssl cmake libcurl4-nss-dev && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*"

ADD . /opt/denonavr

WORKDIR /opt/denonavr

CMD [ 'cargo', 'build' ]

