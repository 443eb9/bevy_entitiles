<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="Tileset1" tilewidth="8" tileheight="8" tilecount="384" columns="12">
 <image source="../Cavernas_by_Adam_Saltsman.png" width="96" height="256"/>
 <tile id="61">
  <animation>
   <frame tileid="61" duration="300"/>
   <frame tileid="72" duration="300"/>
   <frame tileid="73" duration="300"/>
  </animation>
 </tile>
 <tile id="140" type="TileBundle">
  <properties>
   <property name="Infos" type="class" propertytype="TileInfos">
    <properties>
     <property name="AllowLineOfSight" type="bool" value="true"/>
     <property name="DamagePerSecond" type="int" value="50"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="142" type="TileBundle">
  <properties>
   <property name="Infos" type="class" propertytype="TileInfos">
    <properties>
     <property name="DamagePerSecond" type="int" value="50"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="152" type="TileBundle">
  <properties>
   <property name="Infos" type="class" propertytype="TileInfos">
    <properties>
     <property name="AllowLineOfSight" type="bool" value="true"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="154" type="TileBundle"/>
 <tile id="216">
  <animation>
   <frame tileid="217" duration="300"/>
   <frame tileid="216" duration="300"/>
   <frame tileid="218" duration="300"/>
   <frame tileid="219" duration="300"/>
  </animation>
 </tile>
</tileset>
