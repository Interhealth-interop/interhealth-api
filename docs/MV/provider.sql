CREATE OR REPLACE VIEW "DBAMV"."PROVIDER"
(
	"provider_code",
	"provider_name",
	"provider_primary_document",
	"provider_secondary_document",
	"provider_third_document",
    "provider_fourth_document",
    "provider_fifth_document",
    "provider_sixth_document",
	"provider_sex",
	"provider_gender",
	"provider_register",
	"provider_register_uf",
	"provider_register_name",
	"provider_occupation",
	"provider_ddd_phone",
	"provider_phone",
	"provider_ddd_mobile",
	"provider_mobile",
	"provider_ddd_phone_contact",
	"provider_phone_contact",
	"provider_email",
	"provider_birth_date",
	"provider_ishealthprofessional",
	"created_date",
	"created_time",
	"updated_date",
	"updated_time",
	"provider_responsible"
) AS 
SELECT TO_CHAR(P.CD_PRESTADOR) AS "provider_code",
       P.NM_PRESTADOR AS "provider_name",
	   P.NR_CPF_CGC AS "provider_primary_document",
	   P.NR_DOCUMENTO AS "provider_secondary_document",
	   P.NR_CNS AS "provider_third_document",
	   NULL AS "provider_fourth_document",
	   NULL AS "provider_fifth_document",
	   NULL AS "provider_sixth_documento",
	   P.TP_SEXO AS "provider_sex",
	   IG.DS_IDENTIDADE_GENERO AS "provider_gender",
	   P.DS_CODIGO_CONSELHO AS "provider_register",
	   C.CD_UF AS "provider_register_uf",
	   C.DS_CONSELHO AS "provider_register_name",
	   TP.NM_TIP_PRESTA AS "provider_occupation",
	   NULL AS "provider_ddd_phone",
	   NULL AS "provider_phone",
	   NULL AS "provider_ddd_mobile",
	   NULL AS "provider_mobile",
	   NULL AS "provider_ddd_phone_contact",
	   NULL AS "provider_phone_contact",
	   NULL AS "provider_email",
	 
--  (SELECT MAX(NR_DDD_CELULAR||DS_TIP_COMUN_PREST)
--	      FROM DBAMV.PRESTADOR_TIP_COMUN
--		 WHERE CD_PRESTADOR = P.CD_PRESTADOR
--		   AND CD_TIP_COMUN = 1) AS "provider_phone",  -- TROCAR COM BASE NOS CÓDIGOS DO CLIENTE.
--	   (SELECT MAX(NR_DDD_CELULAR||DS_TIP_COMUN_PREST)
--	      FROM DBAMV.PRESTADOR_TIP_COMUN
--		 WHERE CD_PRESTADOR = P.CD_PRESTADOR
--		   AND CD_TIP_COMUN = 3) AS "provider_mobile",  -- TROCAR COM BASE NOS CÓDIGOS DO CLIENTE.
--	   (SELECT MAX(DS_TIP_COMUN_PREST)
--	      FROM DBAMV.PRESTADOR_TIP_COMUN
--		 WHERE CD_PRESTADOR = P.CD_PRESTADOR
--		   AND CD_TIP_COMUN = 7) AS "provider_email",  -- TROCAR COM BASE NOS CÓDIGOS DO CLIENTE.
	
	   TO_CHAR(P.DT_NASCIMENTO, 'YYYY-MM-DD') AS "provider_birth_date",
	   NULL AS "provider_ishealthprofessional",
	   TO_CHAR(P.DT_CADASTRO, 'YYYY-MM-DD') AS "created_date",  
	   TO_CHAR(P.DT_CADASTRO, 'HH24:MI:SS') AS "created_time",  
	   TO_CHAR(UPD.MAX_DH_MODIFICACAO, 'YYYY-MM-DD') AS "updated_date",
       TO_CHAR(UPD.MAX_DH_MODIFICACAO, 'HH24:MI:SS') AS "updated_time",
       UPD.CD_USUARIO_MODIFICACAO AS "provider_responsible"
  FROM DBAMV.PRESTADOR P
  LEFT JOIN DBAMV.CONSELHO C ON P.CD_CONSELHO = C.CD_CONSELHO
  LEFT JOIN DBAMV.IDENTIDADE_GENERO IG ON P.CD_IDENTIDADE_GENERO = IG.CD_IDENTIDADE_GENERO
  LEFT JOIN DBAMV.TIP_PRESTA TP ON P.CD_TIP_PRESTA = TP.CD_TIP_PRESTA
   LEFT JOIN (
       SELECT 
           CD_PRESTADOR,
           MAX(DH_MODIFICACAO) AS MAX_DH_MODIFICACAO, 
           CD_USUARIO_MODIFICACAO
       FROM DBAMV.LOG_PRESTADOR
       GROUP BY CD_PRESTADOR, CD_USUARIO_MODIFICACAO
   ) UPD ON P.CD_PRESTADOR = UPD.CD_PRESTADOR;
