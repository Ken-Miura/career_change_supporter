FROM docker.elastic.co/elasticsearch/elasticsearch:7.13.3
# RUN elasticsearch-plugin remove analysis-kuromoji
# RUN elasticsearch-plugin remove analysis-icu
RUN elasticsearch-plugin install analysis-kuromoji
# TODO: icuプライグインが必要かどうか要確認
#RUN elasticsearch-plugin install analysis-icu
